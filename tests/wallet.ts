import * as anchor from "@com.put/put-anchor";
import { Program } from "@com.put/put-anchor";
import { Wallet } from "../target/types/wallet";

describe("wallet", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Wallet as Program<Wallet>;

  it("Is initialized!", async () => {
    // Add your test here.
    // const tx = await program.methods.getTokenBalance().rpc();
    // console.log("Your transaction signature", tx);
  });
});
