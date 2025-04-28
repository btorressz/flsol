// No imports needed: web3, anchor, pg are globally available in Solana Playground

// Define TOKEN_PROGRAM_ID manually
const TOKEN_PROGRAM_ID = new web3.PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

// Define ASSOCIATED_TOKEN_PROGRAM_ID manually
const ASSOCIATED_TOKEN_PROGRAM_ID = new web3.PublicKey(
  "Add Assoisiate Token Program ID"
);

// Minimal helper: create associated token account manually
async function getOrCreateAssociatedTokenAccount(mint: web3.PublicKey, owner: web3.PublicKey) {
  const [ata] = await web3.PublicKey.findProgramAddressSync(
    [
      owner.toBuffer(),
      TOKEN_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  const accountInfo = await pg.connection.getAccountInfo(ata);
  if (accountInfo === null) {
    const tx = new web3.Transaction().add(
      new web3.TransactionInstruction({
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        keys: [
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
          { pubkey: ata, isSigner: false, isWritable: true },
          { pubkey: owner, isSigner: false, isWritable: false },
          { pubkey: mint, isSigner: false, isWritable: false },
          { pubkey: web3.SystemProgram.programId, isSigner: false, isWritable: false },
          { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
          { pubkey: web3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
        ],
        data: Buffer.alloc(0),
      })
    );
    await pg.connection.sendTransaction(tx, [pg.wallet.keypair]);
  }
  return { address: ata };
}

describe("FLSOL Program Tests", () => {
  let configPda: web3.PublicKey;
  let vaultPda: web3.PublicKey;
  let fsolMintPda: web3.PublicKey;
  let flashRecordPda: web3.PublicKey;
  let userFsolAccount: web3.PublicKey;
  let treasuryWallet: web3.Keypair;

  before(async () => {
    [configPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      pg.program.programId
    );
    [vaultPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), configPda.toBuffer()],
      pg.program.programId
    );
    [fsolMintPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint")],
      pg.program.programId
    );
  });

  it("initialize the program", async () => {
    treasuryWallet = web3.Keypair.generate();

    const tx = await pg.program.methods
      .initialize(
        new BN(5),     // 0.05% fee
        new BN(10000),
        treasuryWallet.publicKey,
        new BN(1),     // 10% to treasury
        new BN(10),
        new BN(web3.LAMPORTS_PER_SOL * 100), // max flash loan 100 SOL
        new BN(100)    // cooldown 100 slots
      )
      .accounts({
        authority: pg.wallet.publicKey,
        config: configPda,
        fsolMint: fsolMintPda,
        vault: vaultPda,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log(`Initialized program. TX = ${tx}`);
  });

  it("stake SOL and mint FLSOL", async () => {
    const ata = await getOrCreateAssociatedTokenAccount(fsolMintPda, pg.wallet.publicKey);
    userFsolAccount = ata.address;

    const stakeAmount = new BN(web3.LAMPORTS_PER_SOL / 10); // 0.1 SOL

    const tx = await pg.program.methods
      .stake(stakeAmount)
      .accounts({
        user: pg.wallet.publicKey,
        userFsolAccount,
        fsolMint: fsolMintPda,
        config: configPda,
        vault: vaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log(`Staked 0.1 SOL. TX = ${tx}`);
  });

  it("harvest rewards (likely 0 early)", async () => {
    const harvestAmount = new BN(100_000_000); // 0.1 SOL worth of FLSOL

    const tx = await pg.program.methods
      .harvest(harvestAmount)
      .accounts({
        user: pg.wallet.publicKey,
        vault: vaultPda,
        config: configPda,
        fsolMint: fsolMintPda,
      })
      .rpc();

    console.log(`Harvested rewards. TX = ${tx}`);
  });

  it("flash loan FLSOL", async () => {
    [flashRecordPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("record"), pg.wallet.publicKey.toBuffer()],
      pg.program.programId
    );

    const flashAmount = new BN(50_000_000); // 0.05 SOL

    const dummyCpiProgram = web3.SystemProgram.programId;

    const tx = await pg.program.methods
      .flashLoan(flashAmount, Buffer.from([]))
      .accounts({
        user: pg.wallet.publicKey,
        userFsolAccount,
        fsolMint: fsolMintPda,
        config: configPda,
        vault: vaultPda,
        flashRecord: flashRecordPda,
        treasuryAccount: treasuryWallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
        receiverProgram: dummyCpiProgram,
      })
      .rpc();

    console.log(`Flash loaned 0.05 SOL worth of FLSOL. TX = ${tx}`);
  });

  it("unstake and withdraw SOL", async () => {
    const unstakeAmount = new BN(100_000_000); // 0.1 SOL worth of FLSOL

    const tx = await pg.program.methods
      .unstake(unstakeAmount)
      .accounts({
        user: pg.wallet.publicKey,
        userFsolAccount,
        fsolMint: fsolMintPda,
        config: configPda,
        vault: vaultPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log(`Unstaked 0.1 SOL. TX = ${tx}`);
  });
});
