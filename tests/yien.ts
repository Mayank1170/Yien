import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Yien } from "../target/types/yien";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("yien", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Yien as Program<Yien>;

  it("Initializes the protocol", async () => {
    const [protocolConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    const [stabilityPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stability_pool")],
      program.programId
    );

    const treasury = anchor.AnchorProvider.env().wallet.publicKey;

    await program.methods
      .initialize({
        minCollateralRatio: new anchor.BN(12000),
        liquidationPenalty: new anchor.BN(500),
        openFee: new anchor.BN(50),
      })
      .accounts({
        protocolConfig: protocolConfigPda,
        stabilityPool: stabilityPoolPda,
        treasury: treasury,
        authority: program.provider.publicKey,
      })
      .rpc();

    const configAccount = await program.account.protocolConfig.fetch(protocolConfigPda);
    console.log("Protocol Config:", configAccount);

    expect(configAccount.minCollateralRatio.toNumber()).to.equal(12000);
    expect(configAccount.openFee.toNumber()).to.equal(50);
  });
});