# SDK Survey — Options Protocols

## Quick Summary

| Protocol | Best SDK | Language | Install | Maintained? | Quality |
|----------|---------|----------|---------|-------------|---------|
| **Deribit** | CCXT | Python/JS/Rust/Go | `pip install ccxt` | Active (daily commits, 41k stars) | Excellent |
| **Derive** | derive-client | Python | `pip install derive-client` | Active (Feb 2026, 1355 commits) | Good |
| **Aevo** | aevo-sdk (official) | Python | Clone repo | Stale (Mar 2024) | Moderate |
| **Premia** | @premia/v3-sdk | TypeScript | `npm i @premia/v3-sdk` | Low activity (Aug 2024) | Alpha |
| **Stryke** | @stryke-xyz/sdk | TypeScript | `npm i @stryke-xyz/sdk` | Low activity (Sep 2025) | Poor (no docs) |

---

## Deribit

### Recommended: CCXT

| Attribute | Value |
|-----------|-------|
| Install | `pip install ccxt` / `npm install ccxt` |
| Stars | 41,300 |
| Last Activity | Daily commits (Mar 2026) |
| Languages | Python, JS/TS, Go, C#, PHP |
| Options Support | Yes — `createOrder`, `fetchOrderBook`, `fetchTicker`, `fetchMarkets` all work with options |

- Unified interface across 100+ exchanges
- Async support via `ccxt.async_support.deribit`
- Limitation: Some niche Deribit features (combos, block trades) may not be exposed

### Alternatives

| SDK | Language | Stars | Last Activity | Install |
|-----|----------|-------|--------------|---------|
| deribit-wrapper | Python | 20 | Sep 2025 (v0.4.2) | `pip install deribit-wrapper` |
| deribit-websocket (joaquinbejar) | Rust | 3 | Mar 2026 | Cargo crate |
| deribit-base (joaquinbejar) | Rust | 5 | Mar 2026 (v0.3.1) | `deribit-base = "0.3"` |
| deribit_websocket_v2 | Python | 47 | Feb 2023 | Clone repo |

**Rust pick**: joaquinbejar ecosystem (deribit-websocket + deribit-base) — production-ready, auto-reconnect, 377 unit tests.

**Deribit-specific Python**: `deribit-wrapper` — clean API, testnet/prod switching, actively maintained.

---

## Derive (formerly Lyra V2)

### Recommended: derive-client (Python)

| Attribute | Value |
|-----------|-------|
| Install | `pip install derive-client` |
| Repo | `8ball030/derive_client` |
| Stars | 9 |
| Last Release | Feb 2026 (v0.3.12) |
| Total Commits | 1,355 |
| Python | 3.10 — 3.13 |

- Most comprehensive: options, perps, spot trading, market data, orders, collateral, bridging
- CLI tool included
- 6 example scripts (basics to bridging)
- Internally uses `derive-action-signing` for on-chain signing
- Community-maintained (not official Derive team), but very active

### Alternatives

| SDK | Language | Status | Notes |
|-----|----------|--------|-------|
| v2-action-signing-python | Python | Moderate (Aug 2025) | Low-level signing only, not a full client |
| cockpit | Rust | Stale (Mar 2024) | Official but unmaintained, no published crate |
| @lyrafinance/lyra-js | JS | Abandoned | V1 only, not compatible with current protocol |

**No maintained JS/TS SDK exists for Derive.**

---

## Aevo

### Recommended: aevo-sdk (Official)

| Attribute | Value |
|-----------|-------|
| Install | Clone `aevoxyz/aevo-sdk` + `pip install -r requirements.txt` |
| Stars | 147 |
| Forks | 69 |
| Last Commit | Mar 2024 |
| Status | Stale (1+ year dormant) |

- WebSocket + REST client
- Order create/edit/cancel (both WS and REST)
- Signing key generation, deposit/withdraw examples
- Dependencies: web3, websockets, aiohttp, eth-account (37 pinned deps)

### Alternatives

| SDK | Language | Stars | Last Activity | Notes |
|-----|----------|-------|--------------|-------|
| aevopy | Python | 14 | Feb 2024 | Cleaner API (`buy_market()`, stop-loss), REST-only, no WS |
| AlethieumAevoSDK | Python | 39 | Nov 2023 | Includes GridBot, heaviest deps |
| aevo-js-sdk | TypeScript | 6 | Feb 2024 | npm v0.0.5, proof of concept, no docs |

**All Aevo SDKs are stale.** For production, consider wrapping the REST/WS API directly using [api-docs.aevo.xyz](https://api-docs.aevo.xyz).

---

## Premia V3

### Recommended: @premia/v3-sdk (TypeScript)

| Attribute | Value |
|-----------|-------|
| Install | `npm i @premia/v3-sdk` (peer dep: ethers v6) |
| Repo | `Premian-Labs/v3-sdk` |
| Stars | 2 |
| Last Publish | Aug 2024 (v2.9.0) |
| Status | Alpha, low activity |

- Full protocol surface: pools, options, pricing, orderbook, vaults, analytics
- API modules: `poolAPI`, `optionAPI`, `ordersAPI`, `pricingAPI`, `vaultAPI`, `analyticsAPI`
- Dependencies: ethers, @apollo/client, @uqee/black-scholes, ws

### Alternatives

| Resource | Type | Last Updated | Notes |
|----------|------|-------------|-------|
| orderbook-api | Docker REST API | Dec 2024 | Language-agnostic HTTP bridge, most recently updated |
| range-order-bot | Example bot | Jan 2024 | 34 stars, good reference for trading bots |
| v3-contracts | Solidity | Jul 2024 | 228 contracts, Forge-based |

**No Python SDK exists.** Options: use `orderbook-api` Docker container as REST bridge, or call contracts directly via web3.py with ABIs from `@premia/v3-abi`.

**Note:** `@premia/v3-sdk-public` does NOT exist on npm (404).

---

## Stryke (formerly Dopex)

### Recommended: @stryke-xyz/sdk (TypeScript)

| Attribute | Value |
|-----------|-------|
| Install | `npm i @stryke-xyz/sdk` |
| Stars | 0 |
| Last Publish | Sep 2025 (v1.0.10) |
| Status | Low activity, zero docs |

- Modules: amms, markets, handlers, hooks, periphery, tokens, chains, ABIs
- Zero runtime dependencies
- **No README, no examples, no documentation** — must read source code

### Alternatives

| Resource | Type | Last Updated | Notes |
|----------|------|-------------|-------|
| premarket-sdk | TypeScript | Feb 2026 | Active internally, not published to npm, no docs |
| clamm (contracts) | Solidity | Apr 2025 | **ARCHIVED**, useful as ABI reference |
| @dopex-io/sdk | TypeScript | May 2023 | **Abandoned** (legacy pre-rebrand) |

**No Python SDK exists.** Would need to wrap contract ABIs via web3.py/viem.

---

## Recommendation by Language

### Python (Recommended for this project)

| Protocol | Package | Readiness |
|----------|---------|-----------|
| Deribit | `pip install ccxt` | Production-ready |
| Derive | `pip install derive-client` | Production-ready |
| Aevo | Clone `aevoxyz/aevo-sdk` | Usable but stale — may need custom wrapper |
| Premia | No Python SDK | Use orderbook-api Docker or direct contract calls |
| Stryke | No Python SDK | Direct contract calls via web3.py |

### TypeScript

| Protocol | Package | Readiness |
|----------|---------|-----------|
| Deribit | `npm install ccxt` | Production-ready |
| Derive | No JS/TS SDK | Must wrap JSON-RPC API directly |
| Aevo | `aevo-js-sdk` (v0.0.5) | Proof of concept only |
| Premia | `npm i @premia/v3-sdk` | Alpha, covers full surface |
| Stryke | `npm i @stryke-xyz/sdk` | No docs, read source code |

### Rust

| Protocol | Package | Readiness |
|----------|---------|-----------|
| Deribit | deribit-websocket + deribit-base | Production-ready, actively maintained |
| Derive | cockpit | Stale (Mar 2024) |
| Aevo | None | Must build custom |
| Premia | None | Must build custom |
| Stryke | None | Must build custom |

---

## Verdict

**Python is the best choice** for this project:
- 2 out of 5 protocols have production-ready Python SDKs (Deribit via CCXT, Derive via derive-client)
- Aevo has a usable but stale Python SDK
- Premia and Stryke would need direct contract interaction (web3.py) or REST wrappers
- Python has the richest ecosystem for options pricing (py_vollib, QuantLib, scipy)
