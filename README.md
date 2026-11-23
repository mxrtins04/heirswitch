# HeirSwitch

**TTL-Aware Dead-Man's Switch & Multi-Heir Self-Custody Recovery Vault on Stellar Soroban**

HeirSwitch protects long-term self-custody Stellar savings from permanent loss. Owners lock USDC/XLM in a personal Soroban vault, configure an inactivity window and multi-heir percentage splits (in basis points), and check in periodically with a single **Heartbeat** transaction.

## Why Stellar & Soroban

- **TTL-aware heartbeat:** Every owner `heartbeat()` call both resets the inactivity countdown **and** explicitly extends the vault's Soroban persistent and instance TTL so long-term savings never face unexpected state archival.
- **Guardian veto protection:** An optional trusted guardian can extend the grace window if the owner is temporarily incapacitated or traveling.
- **Atomic multi-heir distribution:** After the inactivity window elapses, designated beneficiaries claim their exact basis-point share directly on-chain with no central custodian.

## Repository Structure

```text
heirswitch/
├── contracts/heirswitch/   # Rust Soroban smart contract (soroban-sdk 22)
└── web/                    # Consumer web app (Next.js / TypeScript)
```

## Quick Start

```bash
cargo test
cargo build --target wasm32-unknown-unknown --release
```

## License

MIT OR Apache-2.0
