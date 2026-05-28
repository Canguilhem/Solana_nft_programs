import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { NftStaking } from "../target/types/nft_staking";
import { BN } from "bn.js";
import { expect } from "chai";
import { confrimTx, fundWallets } from "./utils";
import { SystemProgram } from "@solana/web3.js";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";

const REWARDS_BPS = 10000;
const FREEZE_PERIOD_IN_DAYS = 7;
const TIME_TRAVEL_IN_DAYS = 8;

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
      throw new Error(`Time travel failed: , ${result.error}`);
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

  it("Mint assets", async () => {});

  it("Stake an asset", async () => {});

  it("Claim rewards", async () => {});

  it("Unstake an asset", async () => {});
});
