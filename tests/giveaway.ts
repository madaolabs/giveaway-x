import * as anchor from "@com.put/put-anchor";
import {
  MessagePrefix,
  concat,
  ethers,
  hashMessage,
  keccak256,
  toUtf8Bytes,
} from "ethers";
import { Program } from "@com.put/put-anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@com.put/ppl-token";
import type { Giveaway } from "../target/types/giveaway";
import { bs58 } from "@com.put/put-anchor/dist/cjs/utils/bytes";

const { PublicKey, SYSVAR_RENT_PUBKEY } = anchor.web3;

const USDT = "USDJASjwdezW9T1oCqDG2ui1PWrTt28QE1s7KBmaids";

describe("Giveaway", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Giveaway as Program<Giveaway>;

  it("createPutGiveaway", async () => {
    const newWallet = ethers.Wallet.createRandom();
    const putGiveawayId = newWallet.address;

    const [giveaway_pool] = await PublicKey.findProgramAddress(
      [ethers.toBeArray(putGiveawayId)],
      program.programId
    );
    try {
      const tx = await program.methods
        .createPutGiveaway({
          giveawayId: Array.from(ethers.toBeArray(putGiveawayId)),
          giveawayCount: 10,
          amount: new anchor.BN(10000000000),
        })
        .accounts({
          payer: provider.wallet.publicKey,
          giveawayPool: giveaway_pool,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .rpc();
      console.log("Your transaction signature", tx);
    } catch (error) {
      console.error(error);
    }
  });

  it("receivePutGiveaway", async () => {
    const phrase =
      "midnight embrace host earn disorder leave twice evolve fresh spot season doll";
    const newWallet = ethers.Wallet.fromPhrase(phrase);
    const putGiveawayId = newWallet.address;

    const [giveaway_pool] = await PublicKey.findProgramAddress(
      [ethers.toBeArray(putGiveawayId)],
      program.programId
    );
    try {
      const receiveAddress = provider.wallet.publicKey;
      const amount = 1000000000;
      const timestamp = new Date().setFullYear(2024);
      const origin_message = ethers.solidityPacked(
        ["address", "bytes32", "uint64", "uint128"],
        [putGiveawayId, receiveAddress.toBytes(), amount, timestamp]
      );
      // const origin_message = putGiveawayId + receiveAddress.toString() + amount + timestamp;
      const signature = await newWallet.signMessage(
        ethers.toBeArray(origin_message)
      );

      const tx = await program.methods
        .receivePutGiveaway({
          giveawayId: Array.from(ethers.toBeArray(putGiveawayId)),
          walletAddress: Array.from<number>(receiveAddress.toBytes()),
          amount: new anchor.BN(amount),
          timestamp: new anchor.BN(timestamp),
          signature: Array.from<number>(ethers.toBeArray(signature)),
        })
        .accounts({
          payer: provider.wallet.publicKey,
          giveawayPool: giveaway_pool,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .rpc();
      console.log("Your transaction signature", tx);
    } catch (error) {
      console.error(error);
    }
  });

  it("createNonPutGiveaway", async () => {
    const newWallet = ethers.Wallet.createRandom();
    const putGiveawayId = newWallet.address;

    const [giveaway_pool] = await PublicKey.findProgramAddress(
      [ethers.toBeArray(putGiveawayId)],
      program.programId
    );

    const [token_pool] = await PublicKey.findProgramAddress(
      [provider.wallet.publicKey.toBytes(), new PublicKey(USDT).toBytes()],
      program.programId
    );

    // 找到一个USDT的关联账户
    const usdt_ass = await getAssociatedTokenAddress(
      new PublicKey(USDT),
      provider.wallet.publicKey
    );

    try {
      const tx = await program.methods
        .createNonPutGiveaway({
          giveawayId: Array.from(ethers.toBeArray(putGiveawayId)),
          giveawayCount: 10,
          amount: new anchor.BN(1000000),
        })
        .accounts({
          systemProgram: SYSTEM_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
          payer: provider.wallet.publicKey,
          giveawayPool: giveaway_pool,
          fromAccount: usdt_ass,
          tokenMint: new PublicKey(USDT),
          tokenPool: token_pool,
        })
        .rpc();
      console.log("Your transaction signature", tx);
    } catch (error) {
      console.error(error);
    }
  });

  it("receiveNonPutGiveaway", async () => {
    const phrase =
      "echo total link boy search leaf arch light rubber able include iron";
    const newWallet = ethers.Wallet.fromPhrase(phrase);
    const putGiveawayId = newWallet.address;

    const [giveaway_pool] = await PublicKey.findProgramAddress(
      [ethers.toBeArray(putGiveawayId)],
      program.programId
    );

    const [token_pool] = await PublicKey.findProgramAddress(
      [provider.wallet.publicKey.toBytes(), new PublicKey(USDT).toBytes()],
      program.programId
    );

    // 找到一个USDT的关联账户
    const usdt_ass = await getAssociatedTokenAddress(
      new PublicKey(USDT),
      provider.wallet.publicKey
    );

    try {
      const amount = 300000;
      const timestamp = parseInt(
        (Number(new Date().setFullYear(2025)) / 1000).toFixed(0)
      );
      // const timestamp = 1710755934;
      const origin_message = ethers.solidityPacked(
        ["bytes32", "address", "uint64", "uint128"],
        [provider.wallet.publicKey.toBytes(), putGiveawayId, timestamp, amount]
      );

      // const origin_message = putGiveawayId + receiveAddress.toString() + amount + timestamp;

      const signature = await newWallet.signingKey.sign(
        ethers.toBeArray(keccak256(origin_message))
      ).serialized;

      const tx = await program.methods
        .receiveNonPutGiveaway({
          giveawayId: Array.from(ethers.toBeArray(putGiveawayId)),
          amount: new anchor.BN(amount),
          timestamp: new anchor.BN(timestamp),
          signature: Array.from<number>(ethers.toBeArray(signature)),
        })
        .accounts({
          payer: provider.wallet.publicKey,
          giveawayPool: giveaway_pool,
          systemProgram: SYSTEM_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMint: new PublicKey(USDT),
          tokenPool: token_pool,
          toAccount: usdt_ass,
        })
        .rpc();
      console.log("Your transaction signature", tx);
    } catch (error) {
      console.error(error);
    }
  });
});
