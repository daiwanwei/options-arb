# Options Arb — Implementation Plan

## Language: Rust

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     options-arb                          │
├─────────────┬──────────────┬────────────────────────────┤
│  Connectors │   Core       │   Execution                │
│             │              │                            │
│  deribit    │  pricing     │  arb-scanner               │
│  derive     │  - BS/Greeks │  - cross-clob IV arb       │
│  aevo       │  - IV solver │  - cefi vs amm vol lag     │
│  premia     │  - vol surf  │  - put-call parity         │
│  stryke     │              │  - 0dte arb                │
│             │  data-model  │  - vol surface arb         │
│             │  - unified   │                            │
│             │    options   │  risk-manager              │
│             │  - orderbook │  - position limits         │
│             │  - trades    │  - greeks limits           │
│             │              │  - kill switch             │
│             │  storage     │                            │
│             │  - timescale │  executor                  │
│             │  - tick data │  - multi-venue order mgmt  │
│             │              │  - fee-aware sizing        │
└─────────────┴──────────────┴────────────────────────────┘
```

## Tech Stack

| Component | Crate |
|-----------|-------|
| Async runtime | `tokio` |
| WebSocket | `tokio-tungstenite` |
| HTTP client | `reqwest` |
| JSON | `serde` / `serde_json` |
| EVM / on-chain | `alloy` |
| Options pricing | `optionstratlib` |
| IV solver | `implied-vol` |
| Vol surface | `volsurf` |
| Storage | `sqlx` (Postgres/TimescaleDB) |
| Logging | `tracing` |
| Config | `config` + `dotenvy` |
| Error handling | `thiserror` / `anyhow` |
| CLI | `clap` |

## Workspace Structure

```
options-arb/
├── Cargo.toml              # workspace root
├── crates/
│   ├── common/             # shared types, error, config
│   ├── pricing/            # BS, Greeks, IV, vol surface
│   ├── connector-deribit/  # Deribit WS + REST client
│   ├── connector-derive/   # Derive JSON-RPC client
│   ├── connector-aevo/     # Aevo REST + WS client
│   ├── connector-premia/   # Premia on-chain (alloy)
│   ├── connector-stryke/   # Stryke CLAMM on-chain (alloy)
│   ├── arb-scanner/        # arb signal detection
│   ├── risk-manager/       # position & exposure limits
│   ├── executor/           # order execution engine
│   └── storage/            # TimescaleDB persistence
├── bin/
│   └── options-arb/        # main binary
├── docs/
│   ├── plan.md
│   ├── deribit-api.md
│   ├── defi-options-protocols.md
│   ├── sdk-survey.md
│   └── rust-options-math.md
└── config/
    ├── default.toml
    └── .env.example
```

## Phases

### Phase 1: Foundation (Milestone 1)

**Goal:** Collect data from Deribit + one DeFi venue, build pricing engine, detect first arb signals.

1. **Workspace scaffolding** — Cargo workspace, common types, config
2. **Unified data model** — Options instrument, orderbook, trade, ticker types that normalize across venues
3. **Pricing engine** — BS pricing, Greeks, IV solver (implied-vol), vol surface builder (volsurf)
4. **Deribit connector** — WebSocket client using `deribit-websocket` crate, subscribe to book/ticker/trades
5. **Derive connector** — JSON-RPC WS client at `wss://api.lyra.finance/ws`, get_all_instruments, get_ticker
6. **Cross-CLOB arb scanner** — Compare IV across Deribit ↔ Derive, flag when spread > threshold after fees
7. **Storage** — TimescaleDB for tick data, arb signals, and PnL tracking

### Phase 2: Multi-Venue (Milestone 2)

**Goal:** Add remaining venues, expand arb strategies.

8. **Aevo connector** — REST (`api.aevo.xyz`) + WS (`ws.aevo.xyz`), orderbook with checksum validation
9. **Premia connector** — On-chain via `alloy` on Arbitrum, query AMM pool quotes, read VolatilitySurfaceOracle
10. **Stryke connector** — On-chain CLAMM contract calls via `alloy`, Option Market + Handler interactions
11. **CeFi vs AMM vol lag scanner** — Deribit IV vs Premia AMM quotes, detect lag during vol spikes
12. **Put-call parity scanner** — Cross-venue parity checks accounting for fees and funding rates
13. **0DTE arb scanner** — Stryke CLAMM premiums vs BS fair value from Deribit IV

### Phase 3: Execution (Milestone 3)

**Goal:** Live trading with risk controls.

14. **Risk manager** — Position limits, net Greeks limits, margin monitoring, kill switch
15. **Executor** — Multi-venue order placement, fee-aware sizing, atomic multi-leg execution
16. **Vol surface arb** — Calendar spread, butterfly, cross-venue skew anomalies
17. **Paper trading mode** — Simulate execution on live data
18. **Monitoring** — Prometheus metrics, Grafana dashboards

## Arb Strategies Summary

| # | Strategy | Venues | Signal |
|---|----------|--------|--------|
| 1 | Cross-CLOB IV | Deribit ↔ Derive ↔ Aevo | Same strike/expiry, IV diff > fees |
| 2 | CeFi vs AMM vol lag | Deribit ↔ Premia | AMM quote lags CLOB IV during vol spikes |
| 3 | Put-call parity | Any pair | C - P ≠ S - K·e^(-rT) across venues |
| 4 | 0DTE arb | Deribit ↔ Stryke | CLAMM mispricing vs BS fair value |
| 5 | Vol surface arb | All venues | Calendar spread, butterfly, skew anomalies |

## Fee Budget

| Venue | Taker Fee | Gas | Notes |
|-------|-----------|-----|-------|
| Deribit | 0.03% of underlying | None | |
| Derive | Variable | Gasless | Off-chain orderbook |
| Aevo | Variable | Gasless | Off-chain orderbook |
| Premia | Taker fee on quote | ~$0.01–0.10 | Arbitrum |
| Stryke | 15% of premium | ~$0.01–0.10 | Arbitrum, massive fee |
