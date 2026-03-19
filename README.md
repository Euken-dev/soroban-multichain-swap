# SoroMultiSwap

> **Atomic batch token swaps across multiple parties on Soroban — price-matched, all-or-nothing.**

A Soroban smart contract that executes a batch of token swaps between multiple parties in a single atomic transaction. Every swap in the batch either succeeds together or the entire transaction reverts — no partial fills, no orphaned transfers.

---

## Hackathon Info

| Field | Details |
|---|---|
| **Track** | DeFi / Finance |
| **Chain** | Stellar (Soroban) |
| **Target users** | DeFi aggregators, batch settlement engines, protocol developers |
| **Status** | Hackathon MVP |

---

## Problem

Executing multiple related token swaps across different parties today requires separate transactions — each with its own authorization, its own risk of partial failure, and no guarantee that the full set of swaps settles together. A failed swap mid-batch leaves counterparties in inconsistent states with no automatic rollback.

---

## Solution

**SoroMultiSwap** wraps a set of individual atomic swaps into a single batched call. It delegates each swap to an underlying `atomic_swap` contract and coordinates authorization across all parties. If any single swap in the batch fails its price check or authorization, the entire batch reverts.

---
### Batch lifecycle

1. **Compose** — Caller assembles a list of swap intents, each specifying two parties, two tokens, amounts, and minimum accepted amounts
2. **Authorize** — All parties in the batch sign off on their respective legs via Soroban's native auth
3. **Dispatch** — SoroMultiSwap iterates the batch and calls the underlying `atomic_swap` contract for each pair
4. **Price check** — Each swap enforces its own `min` threshold; a breach on any swap reverts the entire batch
5. **Settle** — All token transfers across all parties complete atomically in one transaction

---

## Contract API

### Entry points

| Function | Access | Description |
|---|---|---|
| `multi_swap(swaps)` | Any caller | Executes all swaps atomically; reverts entire batch on any failure |

### SwapPair type

```rust
SwapPair {
    a:           Address,   // first party
    b:           Address,   // second party
    token_a:     Address,   // token a sends to b
    token_b:     Address,   // token b sends to a
    amount_a:    i128,      // amount a transfers
    min_b_for_a: i128,      // minimum token_b a will accept
    amount_b:    i128,      // amount b transfers
    min_a_for_b: i128,      // minimum token_a b will accept
}
```

### Authorization

Every party in every `SwapPair` must authorize their leg of the swap. SoroMultiSwap collects and verifies all authorizations before dispatching the batch — no swap is executed until the full batch is authorized.

### Failure behavior

If any swap in the batch fails — price threshold not met, insufficient balance, or missing authorization — the entire transaction reverts. No partial state is written.

---

## Dependencies

This contract requires the compiled `atomic_swap` WASM at build and test time:

```
./atomic_swap/target/wasm32v1-none/release/soroban_atomic_swap_contract.wasm
```

Build the dependency first:

```bash
cd atomic_swap
cargo build --release --target wasm32v1-none
cd ..
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| **Batch contract** | Rust (Soroban) — multi-swap orchestration |
| **Swap contract** | `atomic_swap` WASM — per-pair price enforcement and transfer |
| **Token interface** | SEP-0041 compatible (any Soroban token) |
| **Auth** | Soroban native multi-party authorization across all swap legs |
| **Frontend** | React + Stellar Wallets Kit — batch composer + settlement view |

---

## Why Stellar / Soroban?

- **Cross-contract WASM calls** — SoroMultiSwap delegates to `atomic_swap` natively without bridges or oracles
- **All-or-nothing atomicity** — Soroban's transaction model guarantees full batch revert on any failure
- **Multi-party authorization** — each party signs only their leg; Soroban aggregates auth across the batch
- **SEP-0041 token standard** — any token pair in any swap slot works without custom adapters

---

## Project Structure

```
soroban-atomic-multiswap/
├── atomic_swap/             # Dependency: single-pair atomic swap contract
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── src/
│   ├── lib.rs               # SoroMultiSwap batch orchestration
│   └── test.rs              # Unit tests (swap batching + auth behavior)
├── Cargo.toml
└── README.md
```

---

## Develop

Build the `atomic_swap` dependency first:

```bash
cd atomic_swap && cargo build --release --target wasm32v1-none && cd ..
```

Run tests:

```bash
cargo test
```

Build SoroMultiSwap (WASM):

```bash
cargo build --release --target wasm32v1-none
```

---

## Test Coverage

| # | Scenario | Expected result |
|---|---|---|
| 1 | All swaps meet price thresholds | Full batch settles, all balances updated |
| 2 | One swap breaches min threshold | Entire batch reverts, no state written |
| 3 | Missing authorization on one leg | Batch reverts before any swap executes |
| 4 | Batch with N swap pairs | All N pairs settle atomically |

---

## Hackathon MVP Checklist

- [ ] `multi_swap` entry point accepting a variable-length swap batch
- [ ] Cross-contract delegation to `atomic_swap` WASM per pair
- [ ] Full batch revert on any single swap failure
- [ ] Multi-party authorization collected and verified before dispatch
- [ ] Batch composer UI: add swap pairs, connect wallets per party, preview and submit
- [ ] Live demo: three-party batch swap on Stellar Testnet settling atomically

---

## Differentiators

| Feature | SoroMultiSwap | Sequential swaps |
|---|---|---|
| Atomicity across all pairs | Yes — full revert on failure | No — partial fills possible |
| Single transaction | Yes | No — one tx per swap |
| Multi-party auth | Yes — all legs in one call | No — separate per swap |
| Price protection per leg | Yes — per-pair min threshold | Varies |
| Gas / fee efficiency | One transaction total | N transactions |

---

## Roadmap (Post-Hackathon)

- **V2 — On-chain order matching** — contract matches counterparties from a submitted pool rather than requiring pre-matched pairs
- **V3 — Cross-pool routing** — integrate with SoroPool to route individual legs through AMM liquidity when no direct counterparty is available
- **V4 — Scheduled batches** — submit a batch with a future ledger sequence trigger; executor calls `settle` when the window opens

---

## Contributing

This repo is a hackathon starting point. PRs, issues, and ideas welcome.

---

## License

MIT
