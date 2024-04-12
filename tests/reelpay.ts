import * as anchor from "@com.put/put-anchor";
import { Program } from "@com.put/put-anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@com.put/ppl-token";
import { Reelpay } from "../target/types/reelpay";

const { PublicKey, SYSVAR_RENT_PUBKEY } = anchor.web3;

const USDT = "USDJASjwdezW9T1oCqDG2ui1PWrTt28QE1s7KBmaids";
const BNB = "Bnbic8Saspdw4iSDk1XVNC9sempNmEKJwd9GJfKKKiBx";
const TRX = "TRXJE43EEjo1sWqwqchSTKcG2SiJTAPmeqDQoDhHc5w";
const USDC = "F6xXSEEoSUBHyK28FU338sVpJ4HVeZAG9rLzCtuuuhfR";
const BTC = "3BmGtkTDgQMPoaYvLKTpJV9GvTuFqA7geaTaoejhaGbx";
const ETH = "Ethv6DwMzusT46T332UFFnWiy6qhJXn4TwDehwGAtWed";
const UFO = "DH3Hbn6UyP7J5TMjsjHY3HNHVsWFFP3bV2DwY7ye26sK";

describe("reelpay", async () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Reelpay as Program<Reelpay>;

  // generate put pool
  const [putPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("put_pool")],
    program.programId
  );

  // generate usdt pool
  const [usdtPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("usdt_pool")],
    program.programId
  );

  // generate bnb pool
  const [bnbPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("bnb_pool")],
    program.programId
  );
  // generate btc pool
  const [btcPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("btc_pool")],
    program.programId
  );
  // generate eth pool
  const [ethPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("eth_pool")],
    program.programId
  );
  // generate trx pool
  const [trxPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("trx_pool")],
    program.programId
  );
  // generate usdc pool
  const [usdcPool] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("usdc_pool")],
    program.programId
  );
  // generate usdc pool
  const [admin] = await PublicKey.findProgramAddress(
    [anchor.utils.bytes.utf8.encode("admin")],
    program.programId
  );

  console.log(
    admin.toBase58(),
    putPool.toBase58(),
    btcPool.toBase58(),
    ethPool.toBase58(),
    trxPool.toBase58(),
    bnbPool.toBase58(),
    usdcPool.toBase58(),
    usdtPool.toBase58()
  );
  describe("initialize", async () => {
    it("initialize success!", async () => {
      try {
        const tx = await program.methods
          .initialize({
            admin: provider.wallet.publicKey,
          })
          .accounts({
            payer: provider.wallet.publicKey,
            admin,
            putPool,
            systemProgram: SYSTEM_PROGRAM_ID,
          })
          .rpc();
        console.log("Your transaction signature", tx);
      } catch (error) {
        console.error(error);
      }
    });
  });

  // describe("close", () => {
  //   it("close success!", async () => {
  //     try {
  //       const tx = await program.methods
  //         .close()
  //         .accounts({
  //           payer: provider.wallet.publicKey,
  //           usdtPool,
  //           bnbPool,
  //           btcPool,
  //           ethPool,
  //           trxPool,
  //           usdcPool,
  //           usdtMint: new PublicKey(USDT),
  //           usdcMint: new PublicKey(USDC),
  //           bnbMint: new PublicKey(BNB),
  //           trxMint: new PublicKey(TRX),
  //           ethMint: new PublicKey(ETH),
  //           btcMint: new PublicKey(BTC),
  //           systemProgram: SYSTEM_PROGRAM_ID,
  //           tokenProgram: TOKEN_PROGRAM_ID,
  //           associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //           rent: SYSVAR_RENT_PUBKEY,
  //         })
  //         .rpc();
  //       console.log("Your transaction signature", tx);
  //     } catch (error) {
  //       console.error(error);
  //       throw new Error(error);
  //     }
  //   });
  // });

  describe("pay", async () => {
    it("native", async () => {
      try {
        const tx = await program.methods
          .payNative({
            orderId: "orderId",
            amount: new anchor.BN("100000000"),
          })
          .accounts({
            systemProgram: SYSTEM_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            fromAccount: provider.wallet.publicKey,
            toAccount: putPool,
          })
          .rpc();

        console.log("Your transaction signature", tx);
      } catch (error) {
        console.error(error);
        throw new Error(error);
      }
    });

    it("token", async () => {
      const fromAccount = await getAssociatedTokenAddress(
        new PublicKey(USDT),
        provider.wallet.publicKey
      );
      console.log("from usdt account", fromAccount.toBase58());
      console.log("to account", usdtPool.toBase58());
      try {
        const tx = await program.methods
          .payToken("usdt_pool", {
            orderId: "uuid",
            amount: new anchor.BN("100000"),
          })
          .accounts({
            systemProgram: SYSTEM_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            mintAccount: new PublicKey(USDT),
            payer: provider.wallet.publicKey,
            fromAccount: fromAccount,
            toAccount: usdtPool,
          })
          .rpc();

        console.log("Your transaction signature", tx);
      } catch (error) {
        console.error(error);
        throw new Error(error);
      }
    });
  });

  describe("changeAdmin", () => {
    it("change admin", async () => {
      const tx = await program.methods
        .changeAdmin({
          address: provider.wallet.publicKey,
        })
        .accounts({
          payer: provider.wallet.publicKey,
          admin,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .rpc();

      console.log("Your transaction signature", tx);
    });
  });

  describe("withdraw", () => {
    it("withdraw-token", async () => {
      try {
        const usdtAccount = await getAssociatedTokenAddress(
          new PublicKey(USDT),
          provider.wallet.publicKey
        );
        console.log("usdtAccount--->", usdtAccount.toBase58());
        const tx = await program.methods
          .withdraw({
            payIsMain: false,
            seed: "usdt_pool",
            amount: new anchor.BN("100000"),
          })
          .accounts({
            admin,
            reelpayProgram: program.programId,
            payer: provider.wallet.publicKey,
            systemProgram: SYSTEM_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            fromAccount: usdtPool,
            toAccount: usdtAccount,
          })
          .rpc();
        console.log("Your transaction signature", tx);
      } catch (error) {
        console.error(error);
        throw new Error(error);
      }
    });
    it("withdraw-put", async () => {
      try {
        const tx = await program.methods
          .withdraw({
            payIsMain: true,
            seed: "put_pool",
            amount: new anchor.BN("100000000"),
          })
          .accounts({
            admin,
            reelpayProgram: program.programId,
            payer: provider.wallet.publicKey,
            systemProgram: SYSTEM_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            fromAccount: putPool,
            toAccount: provider.wallet.publicKey,
          })
          .rpc();
        console.log("Your transaction signature", tx);
      } catch (error) {
        console.error(error);
        throw new Error(error);
      }
    });
  });

  describe("createPool", () => {
    it("create rick Pool", async () => {
      try {
        const poolSeed = "usdt_pool";
        // generate rick pool
        const [rickPool] = await PublicKey.findProgramAddress(
          [anchor.utils.bytes.utf8.encode(poolSeed)],
          program.programId
        );
        const tx = await program.methods
          .createPool(poolSeed)
          .accounts({
            systemProgram: SYSTEM_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            payer: provider.wallet.publicKey,
            admin,
            tokenMint: new PublicKey(USDT),
            tokenPool: rickPool,
          })
          .rpc();
        console.log("Your transaction signature", tx);
      } catch (error) {
        console.error(error);
        throw new Error(error);
      }
    });
  });
});
