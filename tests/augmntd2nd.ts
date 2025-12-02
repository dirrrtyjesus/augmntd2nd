import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Augmntd2nd } from "../target/types/augmntd2nd";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("augmntd2nd - Seed 65: The Enharmonic Gap", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.augmntd2nd as Program<Augmntd2nd>;

  // Test accounts
  let mintAuthority: anchor.web3.Keypair;
  let memekMint: anchor.web3.PublicKey;
  let user: anchor.web3.Keypair;
  let userTokenAccount: anchor.web3.PublicKey;
  let seedStatePda: anchor.web3.PublicKey;

  const SEED_ID = new anchor.BN(65);
  const DIFFICULTY = 65;

  before(async () => {
    // 1. Setup Mint Authority and User
    mintAuthority = anchor.web3.Keypair.generate();
    user = anchor.web3.Keypair.generate();

    // Airdrop SOL
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(mintAuthority.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );

    // 2. Create MEMEk Mint
    // Note: In the real program, the seed_state is the mint authority.
    // However, for testing setup, we might need to transfer authority or just mock it.
    // Wait, the program uses seed_state as the mint authority via PDA signer.
    // So we need to create the mint with the seed_state PDA as the authority.

    // Find PDA
    [seedStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("seed_state"), SEED_ID.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    memekMint = await createMint(
      provider.connection,
      mintAuthority, // Payer
      seedStatePda, // Mint Authority (the PDA)
      null, // Freeze Authority
      6 // Decimals
    );

    // 3. Create User Token Account
    userTokenAccount = await createAccount(
      provider.connection,
      user,
      memekMint,
      user.publicKey
    );
  });

  it("Is initialized correctly", async () => {
    await program.methods
      .initialize(SEED_ID, DIFFICULTY)
      .accounts({
        seedState: seedStatePda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await program.account.seed65State.fetch(seedStatePda);
    assert.equal(account.seedId.toNumber(), 65);
    assert.equal(account.difficulty, 65);
    assert.equal(account.isActive, true);
    assert.equal(account.totalBridges.toNumber(), 0);
  });

  it("Pathway A: Bridges via Augmented Second (Exotic)", async () => {
    const completion = {
      context: "B Harmonic Minor",
      intervalName: "Augmented Second",
      resolution: "Resolves upward to E",
      salt: new anchor.BN(12345),
    };

    await program.methods
      .bridgeEnharmonicGap(completion)
      .accounts({
        seedState: seedStatePda,
        user: user.publicKey,
        userTokenAccount: userTokenAccount,
        memekMint: memekMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    // Verify State
    const account = await program.account.seed65State.fetch(seedStatePda);
    assert.equal(account.pathwayACount.toNumber(), 1);
    assert.equal(account.totalBridges.toNumber(), 1);

    // Verify Tokens
    const tokenAccount = await getAccount(provider.connection, userTokenAccount);
    assert.equal(Number(tokenAccount.amount), 65);
  });

  it("Pathway B: Bridges via Minor Third (Stable)", async () => {
    const completion = {
      context: "C Natural Minor",
      intervalName: "Minor Third",
      resolution: "Stable triad tone",
      salt: new anchor.BN(67890),
    };

    await program.methods
      .bridgeEnharmonicGap(completion)
      .accounts({
        seedState: seedStatePda,
        user: user.publicKey,
        userTokenAccount: userTokenAccount,
        memekMint: memekMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    // Verify State
    const account = await program.account.seed65State.fetch(seedStatePda);
    assert.equal(account.pathwayBCount.toNumber(), 1);
    assert.equal(account.totalBridges.toNumber(), 2);

    // Verify Tokens (65 + 65 = 130)
    const tokenAccount = await getAccount(provider.connection, userTokenAccount);
    assert.equal(Number(tokenAccount.amount), 130);
  });

  it("Pathway C: Bridges via Janus Mode (Superposition)", async () => {
    const completion = {
      context: "Schrodinger Superposition",
      intervalName: "Both / Janus Mode",
      resolution: "Context shifts, interval stays",
      salt: new anchor.BN(11111),
    };

    await program.methods
      .bridgeEnharmonicGap(completion)
      .accounts({
        seedState: seedStatePda,
        user: user.publicKey,
        userTokenAccount: userTokenAccount,
        memekMint: memekMint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    // Verify State
    const account = await program.account.seed65State.fetch(seedStatePda);
    assert.equal(account.pathwayCCount.toNumber(), 1);
    assert.equal(account.totalBridges.toNumber(), 3);

    // Verify Tokens (130 + 165 = 295)
    const tokenAccount = await getAccount(provider.connection, userTokenAccount);
    assert.equal(Number(tokenAccount.amount), 295);
  });

  it("Fails on Incoherent Completion", async () => {
    const completion = {
      context: "Random Noise",
      intervalName: "Perfect Fifth", // Wrong interval
      resolution: "Nowhere",
      salt: new anchor.BN(99999),
    };

    try {
      await program.methods
        .bridgeEnharmonicGap(completion)
        .accounts({
          seedState: seedStatePda,
          user: user.publicKey,
          userTokenAccount: userTokenAccount,
          memekMint: memekMint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([user])
        .rpc();
      assert.fail("Should have failed with ContextualIncoherence");
    } catch (e) {
      assert.include(e.message, "ContextualIncoherence");
    }
  });
});
