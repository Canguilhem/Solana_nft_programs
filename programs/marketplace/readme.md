# Marketplace

Anchor program for listing, buying, and offering **MPL Core** NFTs with **native SOL** payments. Each marketplace instance has a configurable fee (basis points), a fee treasury PDA, and a rewards mint for buyers.

**Program ID (localnet):** `GUJJmt5VTpf119kmza9TjRqFW13WXCf5BnMYLMzQHzct`

## Features

| Instruction      | Who signs        | Summary |
|------------------|------------------|---------|
| `initialize`     | Admin            | Create marketplace, treasury PDA, rewards mint (6 decimals). |
| `list`           | Seller (maker)   | List an MPL Core asset; NFT custody moves to the listing PDA. |
| `delist`         | Seller           | Cancel listing; NFT returns to seller; listing account closed. |
| `buy`            | Buyer (taker)    | Pay SOL (net + fee), receive NFT and 1 reward token. |
| `make_offer`     | Buyer (maker)    | Escrow `price` lamports in an offer vault PDA. |
| `cancel_offer`   | Buyer (maker)    | Refund escrow to maker; close offer account. |
| `accept_offer`   | Seller (taker)   | Seller accepts offer: NFT → maker, escrow → seller (net) + treasury (fee), buyer gets reward. |
| `withdraw_fee`   | Admin            | Withdraw accumulated fees from treasury (keeps rent-exempt minimum). |

Payments and offers use **lamports only** (system program transfers). SPL-token payments are not implemented yet.

## Fee model

Fees are in **basis points** (10_000 = 100%). Max fee at init is 10_000.

```text
fee_lamports = price * fee_bps / 10_000
net_lamports = price - fee_lamports
```

Used on `buy` and `accept_offer`: buyer/taker pays `price`; seller receives `net`; treasury receives `fee`.

Helper in tests (300 bps = 3%):

```rust
marketplace::split_price(price, fee_bps) -> Result<(fee, net)>
```

## Rewards

On `buy` and `accept_offer`, the **taker** (buyer on list flow, seller on accept-offer flow) receives **1** token from the marketplace rewards mint via `mint_to`. The marketplace PDA is mint authority.

## Accounts & PDAs

| Seed            | Account        | Notes |
|-----------------|----------------|-------|
| `marketplace` + name | `Marketplace` | Admin, fee bps, bumps, name (max 32 chars). |
| `treasury` + marketplace | System PDA | Holds protocol fees. |
| `REWARDS` + marketplace | SPL mint | Rewards token; authority = marketplace. |
| `listing` + asset | `Listing` | One active listing per asset. |
| `offer` + asset + maker | `Offer` | Offer metadata; closed on cancel/accept. |
| `offer_vault` + asset + maker | System PDA | Escrow for offer lamports. |

### State layouts

- **Marketplace:** `admin`, `fee`, `bump`, `treasury_bump`, `rewards_bump`, `name`
- **Listing:** `maker`, `asset`, `price`, `bump`
- **Offer:** `maker`, `asset`, `price`, `bump`, `vault_bump`

## Flows

### List → buy

1. Seller calls `list` — NFT owner becomes the listing PDA.
2. Buyer calls `buy` — SOL: taker → maker (net) + treasury (fee); NFT: listing → taker; rewards minted to taker ATA.

### Offer → accept / cancel

1. Buyer calls `make_offer` — `price` lamports: maker → `offer_vault`; `Offer` account created.
2. **Cancel:** maker calls `cancel_offer` — full escrow refunded; offer closed.
3. **Accept:** NFT owner calls `accept_offer` — NFT: taker (seller) → maker (buyer); escrow: vault → taker (net) + treasury (fee); offer closed; reward to taker.

Naming in `accept_offer`: **`taker`** is the NFT owner accepting (seller); **`maker`** is the offer bidder (buyer).

## Project layout

```text
programs/marketplace/
├── src/
│   ├── lib.rs              # Program entrypoints
│   ├── constants.rs        # PDA seeds
│   ├── error.rs
│   ├── state/              # Marketplace, Listing, Offer
│   └── instructions/       # Per-instruction account structs & logic
└── tests/
    ├── tests.rs            # Integration tests (#[test] only)
    ├── fixtures/           # mpl_core.so (required for LiteSVM tests)
    └── common/             # Shared helpers (not separate test binaries)
        ├── mod.rs
        ├── constants.rs    # MPL_CORE_ID, seeds
        ├── instructions.rs # IX builders for tests
        └── setup.rs        # LiteSVM deploy, init, NFT mint helpers
```

Cargo compiles every top-level `tests/*.rs` as its own integration-test binary. Shared code lives under `tests/common/` and is included via `mod common;` from `tests.rs` only ([Cargo test guide](https://doc.rust-lang.org/cargo/guide/tests.html#sharing-code-between-tests)).

## Build & test

From the workspace root:

```bash
# Build the program (required before LiteSVM tests)
anchor build -p marketplace

# Run all marketplace integration tests
cargo test -p marketplace --test tests

# Single test
cargo test -p marketplace --test tests -- test_buy --exact --nocapture
```

### Test prerequisites

1. **`target/deploy/marketplace.so`** — produced by `anchor build`.
2. **`tests/fixtures/mpl_core.so`** — MPL Core program binary for LiteSVM (clone from mainnet or your build). Mainnet program id: `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`.

Tests use [anchor-litesvm](https://crates.io/crates/anchor-litesvm) (no validator). MPL Core collection/asset creation uses `CreateCollectionV2` / `CreateV2` in `common/setup.rs`.

### Integration tests

| Test                      | Covers |
|---------------------------|--------|
| `test_initialize_marketplace` | `initialize`, marketplace account |
| `test_listing`            | `list`, listing + NFT custody |
| `test_buy`                | `buy`, ownership, balance logging |
| `cancel_listing`          | `delist`, listing closed, NFT back to seller |
| `test_make_offer`         | `make_offer`, escrow debited from maker |
| `test_cancel_offer`       | `cancel_offer`, vault drained, offer closed, refund |
| `test_accept_offer`       | `accept_offer`, NFT to maker, seller paid net |
| `test_withdraw_fee`       | `buy` fee in treasury, `withdraw_fee` to admin |

Default test marketplace: name `"test"`, fee **300 bps (3%)**. Helpers: `init_marketplace`, `get_pdas`, `expected_split`, `assert_nft_owner`.

## Errors

| Code | When |
|------|------|
| `InvalidFeeAmount` | Fee bps > 10_000 at init |
| `InvalidPrice` | Price must be > 0 for list/offer |
| `MathError` | Overflow or treasury withdraw math |
| `Unauthorized` | Non-admin `withdraw_fee` |

## Roadmap / limitations

- [ ] SPL-token payments (offers and listings still SOL-only)
- [ ] Anchor TS tests / client SDK in workspace `tests/`
- [ ] Stricter balance assertions in integration tests (tx fees, rent on offer/listing close)

## Dependencies

- Anchor `0.31.1`
- `mpl-core` `0.11.1` (anchor feature)
- Dev: `anchor-litesvm` `0.2.0`
