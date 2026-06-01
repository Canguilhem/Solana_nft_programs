import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { NftStaking } from "../target/types/nft_staking";
import { BN } from "bn.js";
import { expect } from "chai";
import { confrimTx, fundWallets } from "./utils";
import { SystemProgram } from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { token } from "@anchor-lang/core/dist/cjs/utils";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const REWARDS_BPS = 10000;
const FREEZE_PERIOD_IN_DAYS = 7;
const TIME_TRAVEL_IN_DAYS = 8;
const MILLISECONDS_PER_DAY = 1000 * 60 * 60 * 24;

describe("staking tests", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.nft_staking as Program<NftStaking>;

  const collectionKeypair = anchor.web3.Keypair.generate();
  const nftKeypair = anchor.web3.Keypair.generate();

  // derive PDAs
  const updateAuthority = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("update_authority"), collectionKeypair.publicKey.toBuffer()],
    program.programId,
  )[0];

  const config = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config"), collectionKeypair.publicKey.toBuffer()],
    program.programId,
  )[0];

  const rewardsMint = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("rewards_mint"), collectionKeypair.publicKey.toBuffer()],
    program.programId,
  )[0];

  const userRewardsAta = getAssociatedTokenAddressSync(
    rewardsMint,
    provider.wallet.publicKey,
    false,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );

  async function advanceTime(params: {
    absoluteEpoch?: number;
    absoluteSlot?: number;
    absoluteTimestamp?: number;
  }): Promise<void> {
    const rpcRes = await fetch(provider.connection.rpcEndpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "surfnet_timeTravel",
        params: [params],
      }),
    });
    const result = (await rpcRes.json()) as { error?: any; result?: any };
    if (result.error) {
      throw new Error(`Time travel failed: , ${JSON.stringify(result.error)}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 1000));
  }

  before(async () => {});

  it("Create a collection", async () => {
    const collectionName = "Test collection";
    const collectionUri = "https://mock.com";
    const tx = await program.methods
      .createCollection(collectionName, collectionUri)
      .accountsPartial({
        payer: provider.wallet.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([collectionKeypair])
      .rpc();

    console.log("signature: ", tx);
    console.log("collection address: ", collectionKeypair.publicKey.toBase58());
  });

  it("Initialize", async () => {
    const tx = await program.methods
      .initialize(REWARDS_BPS, FREEZE_PERIOD_IN_DAYS)
      .accountsPartial({
        admin: provider.wallet.publicKey,
        config,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        rewardsMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("init signature: ", tx);
  });

  it("Mint assets", async () => {
    const tx = await program.methods
      .mintAsset("test token", "https://example.com")
      .accountsStrict({
        user: provider.wallet.publicKey,
        asset: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .signers([nftKeypair])
      .rpc();

    console.log("signature: ", tx);
    console.log("nft address: ", nftKeypair.publicKey.toBase58());
  });

  it("Stake an asset", async () => {
    const tx = await program.methods
      .stake()
      .accountsStrict({
        owner: provider.wallet.publicKey,
        config,
        asset: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      // .signers([nftKeypair])
      .rpc();

    console.log("staking signature: ", tx);
  });

  it("Tries to unstake before allowed", async () => {
    try {
      const tx = await program.methods
        .unstake()
        .accountsStrict({
          owner: provider.wallet.publicKey,
          config,
          asset: nftKeypair.publicKey,
          collection: collectionKeypair.publicKey,
          updateAuthority,
          rewardsMint,
          userRewardsAta,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .rpc();

      throw new Error(
        "Unstake should have failed before freeze period elapsed but tx succeeded",
      );
    } catch (error) {
      if (
        error instanceof anchor.AnchorError &&
        error.error.errorCode.code == "FreezePeriodNotElapsed"
      ) {
        console.log("Unstake failed as expected: ", error.error.errorMessage);
      } else {
        throw error;
      }
    }
  });

  it("Time Travel", async () => {
    const slot = await provider.connection.getSlot();
    const currentSec = (await provider.connection.getBlockTime(slot))!;

    await advanceTime({
      absoluteTimestamp:
        currentSec * 1000 + TIME_TRAVEL_IN_DAYS * MILLISECONDS_PER_DAY,
    });

    const slot2 = await provider.connection.getSlot();
    const currentSec2 = (await provider.connection.getBlockTime(slot2))!;

    console.log(`jumped from ${slot} -> ${slot2}`);
  });

  it("Claim rewards", async () => {
    const tx = await program.methods
      .claimRewards()
      .accountsStrict({
        owner: provider.wallet.publicKey,
        config,
        asset: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        rewardsMint,
        userRewardsAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("claiming signature: ", tx);
  });

  it("Unstake an asset", async () => {
    const tx = await program.methods
      .unstake()
      .accountsStrict({
        owner: provider.wallet.publicKey,
        config,
        asset: nftKeypair.publicKey,
        collection: collectionKeypair.publicKey,
        updateAuthority,
        rewardsMint,
        userRewardsAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        mplCoreProgram: MPL_CORE_PROGRAM_ID,
      })
      .rpc();
    console.log("unstake signature: ", tx);
  });
});
