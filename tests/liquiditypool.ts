import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LiquidityPoolProject } from "../target/types/liquidity_pool_project";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

describe("liquidity_pool_project", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.liquidityPoolProject as Program<LiquidityPoolProject>;

  let mintA: PublicKey;
  let mintB: PublicKey;
  let lpTokenMint: PublicKey;
  let poolConfigAccount: PublicKey;
  let vaultTokenA: PublicKey;
  let vaultTokenB: PublicKey;
  let creatorTokenAccount: PublicKey;
  let lpMintAuthority: PublicKey;
  let poolAuthority: PublicKey;

  before(async () => {
    mintA = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      1
    );

    mintB = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      1
    );

    [poolConfigAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("poolconfig"), mintA.toBuffer(), mintB.toBuffer()],
      program.programId
    );

    const vaultTokenAata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mintA,
      poolConfigAccount,
      true
    );
    vaultTokenA = vaultTokenAata.address;

    const vaultTokenBata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mintB,
      poolConfigAccount,
      true
    );
    vaultTokenB = vaultTokenBata.address;

    [poolAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_authority"), poolConfigAccount.toBuffer()],
      program.programId
    );

    [lpMintAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp_mint")],
      program.programId
    );

    lpTokenMint = await createMint(
      provider.connection,
      provider.wallet.payer,
      lpMintAuthority,
      null,
      1
    );

    const creatorTokenAccountATA = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      lpTokenMint,
      provider.wallet.publicKey,
      true
    );
    creatorTokenAccount = creatorTokenAccountATA.address;
  });

  it("Initialize liquidity pool", async () => {
    const fees = 0.5;
    const tx = await program.methods
      .initializeLiquidityPool(fees)
      .accountsPartial({
        creator: provider.wallet.publicKey,
        mintA,
        mintB,
        lpMint: lpTokenMint,
        poolConfigAccount,
        vaultTokenA,
        vaultTokenB,
        creatorTokenAccount,
        lpMintAuth: lpMintAuthority,
        poolAuthority,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([provider.wallet.payer])
      .rpc();

    console.log(`Transaction Signature: ${tx}`);
  });

  it("Deposit liquidity", async () => {
  // Create user token accounts for deposits
  const userTokenAccountA = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mintA,
    provider.wallet.publicKey,
    true
  );

  const userTokenAccountB = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mintB,
    provider.wallet.publicKey,
    true
  );

  // Create user LP token account
  const userLpTokenAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    lpTokenMint,
    provider.wallet.publicKey,
    true
  );

  // Mint some tokens to user for testing
  const { mintTo } = await import("@solana/spl-token");
  
  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mintA,
    userTokenAccountA.address,
    provider.wallet.publicKey,
    100 * 10 ** 1 // 100 tokens with 1 decimal
  );

  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mintB,
    userTokenAccountB.address,
    provider.wallet.publicKey,
    2000 * 10 ** 1 // 2000 tokens with 1 decimal
  );

  // Mock Pyth price feed accounts (you'll need to replace with actual Pyth accounts)
  // For testing, you might want to create mock accounts or use devnet Pyth feeds
  const mockPriceFeedA = new PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix"); // SOL/USD
  const mockPriceFeedB = new PublicKey("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD"); // USDC/USD

  // Deposit parameters
  const amountA = 10 * 10 ** 1; // 10 tokens A
  const amountB = 200 * 10 ** 1; // 200 tokens B
  const minLpTokens = 1; // Minimum LP tokens expected
  const priceFeedIdA = "0xe62df6c8b4c85fe1fccc88e45a692068c28311c3f20c0968adae46094b6e3c68"; // SOL/USD feed ID
  const priceFeedIdB = "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a"; // USDC/USD feed ID

  console.log("Depositing liquidity...");
  console.log(`Amount A: ${amountA}, Amount B: ${amountB}`);

  const tx = await program.methods
    .depositLiquidityPool(
      new anchor.BN(amountA),
      new anchor.BN(amountB),
      new anchor.BN(minLpTokens),
      priceFeedIdA,
      priceFeedIdB
    )
    .accountsPartial({
      user: provider.wallet.publicKey,
      poolConfig: poolConfigAccount,
      mintA,
      mintB,
      lpMint: lpTokenMint,
      vaultTokenA,
      vaultTokenB,
      userTokenA: userTokenAccountA.address,
      userTokenB: userTokenAccountB.address,
      userLpToken: userLpTokenAccount.address,
      priceFeedA: mockPriceFeedA,
      priceFeedB: mockPriceFeedB,
      lpMintAuth: lpMintAuthority,
      poolAuthority,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([provider.wallet.payer])
    .rpc();

  console.log(`Deposit Transaction Signature: ${tx}`);

  // Verify the deposit
  const poolConfigData = await program.account.liquidityPoolConfig.fetch(poolConfigAccount);
  console.log("Pool config after deposit:", {
    tokenADeposits: poolConfigData.tokenADeposits.toString(),
    tokenBDeposits: poolConfigData.tokenBDeposits.toString(),
    totalPoolValue: poolConfigData.totalPoolValue.toString(),
  });

  // Check user token balances
  const userTokenABalance = await provider.connection.getTokenAccountBalance(userTokenAccountA.address);
  const userTokenBBalance = await provider.connection.getTokenAccountBalance(userTokenAccountB.address);
  const userLpBalance = await provider.connection.getTokenAccountBalance(userLpTokenAccount.address);

  console.log("User balances after deposit:", {
    tokenA: userTokenABalance.value.uiAmount,
    tokenB: userTokenBBalance.value.uiAmount,
    lpTokens: userLpBalance.value.uiAmount,
  });

  // Check vault balances
  const vaultABalance = await provider.connection.getTokenAccountBalance(vaultTokenA);
  const vaultBBalance = await provider.connection.getTokenAccountBalance(vaultTokenB);

  console.log("Vault balances after deposit:", {
    vaultA: vaultABalance.value.uiAmount,
    vaultB: vaultBBalance.value.uiAmount,
  });

  
});

it("Deposit liquidity - second user", async () => {
  // Create a second user keypair
  const secondUser = anchor.web3.Keypair.generate();
  
  // Airdrop some SOL to the second user
  await provider.connection.requestAirdrop(secondUser.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
  await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for airdrop

  // Create token accounts for second user
  const secondUserTokenAccountA = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mintA,
    secondUser.publicKey,
    true
  );

  const secondUserTokenAccountB = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mintB,
    secondUser.publicKey,
    true
  );

  const secondUserLpTokenAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    lpTokenMint,
    secondUser.publicKey,
    true
  );

  // Mint tokens to second user
  const { mintTo } = await import("@solana/spl-token");
  
  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mintA,
    secondUserTokenAccountA.address,
    provider.wallet.publicKey,
    50 * 10 ** 1 // 50 tokens A
  );

  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mintB,
    secondUserTokenAccountB.address,
    provider.wallet.publicKey,
    1000 * 10 ** 1 // 1000 tokens B
  );

  // Mock Pyth price feeds (same as before)
  const mockPriceFeedA = new PublicKey("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix");
  const mockPriceFeedB = new PublicKey("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD");

  const amountA = 5 * 10 ** 1; // 5 tokens A
  const amountB = 100 * 10 ** 1; // 100 tokens B
  const minLpTokens = 1;
  const priceFeedIdA = "0xe62df6c8b4c85fe1fccc88e45a692068c28311c3f20c0968adae46094b6e3c68";
  const priceFeedIdB = "0xeaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a";

  console.log("Second user depositing liquidity...");

  const tx = await program.methods
    .depositLiquidityPool(
      new anchor.BN(amountA),
      new anchor.BN(amountB),
      new anchor.BN(minLpTokens),
      priceFeedIdA,
      priceFeedIdB
    )
    .accountsPartial({
      user: secondUser.publicKey,
      poolConfig: poolConfigAccount,
      mintA,
      mintB,
      lpMint: lpTokenMint,
      vaultTokenA,
      vaultTokenB,
      userTokenA: secondUserTokenAccountA.address,
      userTokenB: secondUserTokenAccountB.address,
      userLpToken: secondUserLpTokenAccount.address,
      priceFeedA: mockPriceFeedA,
      priceFeedB: mockPriceFeedB,
      lpMintAuth: lpMintAuthority,
      poolAuthority,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([secondUser])
    .rpc();

  console.log(`Second deposit Transaction Signature: ${tx}`);

  // Verify pool state after second deposit
  const poolConfigData = await program.account.liquidityPoolConfig.fetch(poolConfigAccount);
  console.log("Pool config after second deposit:", {
    tokenADeposits: poolConfigData.tokenADeposits.toString(),
    tokenBDeposits: poolConfigData.tokenBDeposits.toString(),
    totalPoolValue: poolConfigData.totalPoolValue.toString(),
  });

  // Check LP token supply
  const lpMintInfo = await provider.connection.getTokenSupply(lpTokenMint);
  console.log("Total LP token supply:", lpMintInfo.value.uiAmount);
});

});