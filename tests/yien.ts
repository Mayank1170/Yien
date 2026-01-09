import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Yien } from "../target/types/yien";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount, getAssociatedTokenAddress, createAssociatedTokenAccountInstruction, createInitializeAccountInstruction, getMinimumBalanceForRentExemptAccount } from "@solana/spl-token";
import { Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { expect } from "chai";

describe("yien", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Yien as Program<Yien>;

  let mint: PublicKey;
  let userTokenAccount: PublicKey;
  let vaultTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;

  const payer = (provider.wallet as any).payer || provider.wallet;

  before(async () => {
    mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      6
    );

    userTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey
    );

    await mintTo(
      provider.connection,
      payer,
      mint,
      userTokenAccount,
      payer,
      1000_000_000
    );

    // Derive PDA addresses - these will be the token account addresses
    // The program expects token accounts at these PDA addresses
    [vaultTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), mint.toBuffer()],
      program.programId
    );

    [treasuryTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury_vault"), mint.toBuffer()],
      program.programId
    );

    // Note: The token accounts at these PDA addresses will be created
    // by the program when openPosition is called, or we need to create
    // them manually. For now, the program should handle creation.
  });

  it("Initializes the protocol", async () => {
    const [protocolConfigPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    const [stabilityPoolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stability_pool")],
      program.programId
    );

    await program.methods
      .initialize({
        minCollateralRatio: new anchor.BN(12000),
        liquidationPenalty: new anchor.BN(500),
        openFee: new anchor.BN(50),
      })
      .accounts({
        protocolConfig: protocolConfigPda,
        stabilityPool: stabilityPoolPda,
        treasury: payer.publicKey,
        authority: payer.publicKey,
      })
      .rpc();

    const config = await program.account.protocolConfig.fetch(protocolConfigPda);
    console.log("Config:", config);
  });

  it("Opens a farm position successfully", async () => {
    const [positionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("position"), payer.publicKey.toBuffer(), mint.toBuffer()],
      program.programId
    );

    const amount = new anchor.BN(100_000_000);

    await program.methods
      .openPosition(amount)
      .accounts({
        user: payer.publicKey,
        farmPosition: positionPda,
        collateralMint: mint,
        userCollateralAccount: userTokenAccount,
        vaultCollateralAccount: vaultTokenAccount,
        treasuryCollateralAccount: treasuryTokenAccount,
        protocolConfig: await getConfigPda(),
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const position = await program.account.farmPosition.fetch(positionPda);
    console.log("Position:", position);

    expect(position.collateralAmount.toNumber()).to.equal(99_500_000);

    const vaultAcc = await getAccount(provider.connection, vaultTokenAccount);
    expect(vaultAcc.amount.toString()).to.equal("100000000");

    const treasuryAcc = await getAccount(provider.connection, treasuryTokenAccount);
    expect(treasuryAcc.amount.toString()).to.equal("500000");
  });

  async function getConfigPda(): Promise<PublicKey> {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    )[0];
  }
});