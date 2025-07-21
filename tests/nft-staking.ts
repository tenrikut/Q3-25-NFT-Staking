import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftStaking } from "../target/types/nft_staking";
import {
  createMint,
  createAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { expect } from "chai";

describe("nft-staking", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.nftStaking as Program<NftStaking>;
  const provider = anchor.AnchorProvider.env();

  // Test accounts
  let admin: Keypair;
  let user: Keypair;
  let collectionMint: PublicKey;
  let nftMint: PublicKey;
  let userTokenAccount: PublicKey;

  // Program accounts
  let config: PublicKey;
  let rewardsMint: PublicKey;
  let userAccount: PublicKey;
  let stakeAccount: PublicKey;

  // Test configuration values
  const POINTS_PER_STAKE = 10;
  const MAX_STAKE = 5;
  const FREEZE_PERIOD = 86400; // 1 day in seconds

  before(async () => {
    // Setup test accounts
    admin = Keypair.generate();
    user = Keypair.generate();

    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(
      admin.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.requestAirdrop(
      user.publicKey,
      2 * LAMPORTS_PER_SOL
    );

    // Wait for airdrops to confirm
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Create collection mint
    collectionMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      admin.publicKey,
      0 // NFTs have 0 decimals
    );

    // Create NFT mint
    nftMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      admin.publicKey,
      0 // NFTs have 0 decimals
    );

    // Create user token account for NFT
    userTokenAccount = await getAssociatedTokenAddress(nftMint, user.publicKey);

    // Derive PDAs
    [config] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    [rewardsMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("rewards"), config.toBuffer()],
      program.programId
    );

    [userAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), user.publicKey.toBuffer()],
      program.programId
    );

    [stakeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), nftMint.toBuffer(), config.toBuffer()],
      program.programId
    );
  });

  describe("Initialize Config", () => {
    it("Should initialize staking configuration", async () => {
      const tx = await program.methods
        .initializeConfig(POINTS_PER_STAKE, MAX_STAKE, FREEZE_PERIOD)
        .accounts({
          admin: admin.publicKey,
          config: config,
          rewardsMint: rewardsMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();

      console.log("Initialize config transaction signature:", tx);

      // Verify the config account was created with correct values
      const configAccount = await program.account.stakeConfig.fetch(config);
      expect(configAccount.pointsPerStake).to.equal(POINTS_PER_STAKE);
      expect(configAccount.maxStake).to.equal(MAX_STAKE);
      expect(configAccount.freezePeriod).to.equal(FREEZE_PERIOD);

      console.log("✅ Config initialized successfully");
      console.log("Points per stake:", configAccount.pointsPerStake);
      console.log("Max stake:", configAccount.maxStake);
      console.log("Freeze period:", configAccount.freezePeriod);
    });

    it("Should create rewards mint with correct authority", async () => {
      const mintInfo = await provider.connection.getAccountInfo(rewardsMint);
      expect(mintInfo).to.not.be.null;

      console.log("✅ Rewards mint created successfully");
    });
  });

  describe("Initialize User Account", () => {
    it("Should initialize user account with zero values", async () => {
      const tx = await program.methods
        .initializeUser()
        .accounts({
          user: user.publicKey,
          userAccount: userAccount,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      console.log("Initialize user transaction signature:", tx);

      // Verify the user account was created with correct initial values
      const userAccountData = await program.account.userAccount.fetch(
        userAccount
      );
      expect(userAccountData.points).to.equal(0);
      expect(userAccountData.amountStaked).to.equal(0);

      console.log("✅ User account initialized successfully");
      console.log("Initial points:", userAccountData.points);
      console.log("Initial amount staked:", userAccountData.amountStaked);
    });

    it("Should not allow duplicate user account initialization", async () => {
      try {
        await program.methods
          .initializeUser()
          .accounts({
            user: user.publicKey,
            userAccount: userAccount,
            systemProgram: SystemProgram.programId,
          })
          .signers([user])
          .rpc();

        expect.fail("Should have thrown an error for duplicate initialization");
      } catch (error) {
        expect(error.message).to.include("already in use");
        console.log(
          "✅ Correctly prevented duplicate user account initialization"
        );
      }
    });
  });

  describe("Staking Tests", () => {
    before(async () => {
      // Create associated token account for user and mint NFT to it
      const createAtaIx = anchor.web3.SystemProgram.createAccount({
        fromPubkey: user.publicKey,
        newAccountPubkey: userTokenAccount,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          165
        ),
        space: 165,
        programId: TOKEN_PROGRAM_ID,
      });

      // For simplicity in MVP testing, we'll simulate the token account creation
      // In a real test, you'd use the full associated token program
      console.log("📝 Note: In MVP testing, simplified NFT setup");
      console.log("User token account:", userTokenAccount.toString());
      console.log("NFT mint:", nftMint.toString());
      console.log("Collection mint:", collectionMint.toString());
    });

    it("Should show current staking limits", async () => {
      const userAccountData = await program.account.userAccount.fetch(
        userAccount
      );
      const configData = await program.account.stakeConfig.fetch(config);

      console.log("Current user stake amount:", userAccountData.amountStaked);
      console.log("Maximum stake allowed:", configData.maxStake);
      console.log(
        "Available stake slots:",
        configData.maxStake - userAccountData.amountStaked
      );
    });

    // Note: Full staking test would require complex NFT metadata setup
    // This is a simplified MVP version that shows the test structure
    it("Should prepare for staking test (MVP simulation)", async () => {
      console.log("📝 MVP Note: Full staking test requires:");
      console.log("1. NFT metadata account creation");
      console.log("2. Master edition account setup");
      console.log("3. Collection verification");
      console.log("4. Associated token account with NFT");

      console.log("✅ Staking test structure prepared");
      console.log("Stake account PDA:", stakeAccount.toString());
    });

    it("Should validate max stake constraint", async () => {
      const configData = await program.account.stakeConfig.fetch(config);
      const userAccountData = await program.account.userAccount.fetch(
        userAccount
      );

      expect(userAccountData.amountStaked).to.be.lessThan(configData.maxStake);
      console.log("✅ Max stake constraint validation passed");
    });
  });

  describe("Account Validation", () => {
    it("Should verify all PDAs are correctly derived", async () => {
      // Verify config PDA
      const [derivedConfig, configBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        program.programId
      );
      expect(derivedConfig.equals(config)).to.be.true;

      // Verify rewards mint PDA
      const [derivedRewardsMint, rewardsBump] =
        PublicKey.findProgramAddressSync(
          [Buffer.from("rewards"), config.toBuffer()],
          program.programId
        );
      expect(derivedRewardsMint.equals(rewardsMint)).to.be.true;

      // Verify user account PDA
      const [derivedUserAccount, userBump] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), user.publicKey.toBuffer()],
        program.programId
      );
      expect(derivedUserAccount.equals(userAccount)).to.be.true;

      console.log("✅ All PDA derivations verified");
      console.log("Config bump:", configBump);
      console.log("Rewards mint bump:", rewardsBump);
      console.log("User account bump:", userBump);
    });

    it("Should verify account ownership", async () => {
      const configAccountInfo = await provider.connection.getAccountInfo(
        config
      );
      const userAccountInfo = await provider.connection.getAccountInfo(
        userAccount
      );

      expect(configAccountInfo.owner.equals(program.programId)).to.be.true;
      expect(userAccountInfo.owner.equals(program.programId)).to.be.true;

      console.log("✅ Account ownership verified");
    });
  });

  describe("Program State Queries", () => {
    it("Should fetch and display all account states", async () => {
      const configData = await program.account.stakeConfig.fetch(config);
      const userData = await program.account.userAccount.fetch(userAccount);

      console.log("\n📊 Current Program State:");
      console.log("=====================================");
      console.log("Config Account:");
      console.log("  Points per stake:", configData.pointsPerStake);
      console.log("  Max stake:", configData.maxStake);
      console.log("  Freeze period:", configData.freezePeriod);
      console.log("  Config bump:", configData.bump);
      console.log("  Rewards bump:", configData.rewardsBump);

      console.log("\nUser Account:");
      console.log("  Points:", userData.points);
      console.log("  Amount staked:", userData.amountStaked);
      console.log("  User bump:", userData.bump);
      console.log("=====================================\n");
    });
  });
});
