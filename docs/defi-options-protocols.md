# DeFi Options Protocols Reference

## Protocol Landscape

| Protocol | Chain | Model | Option Style | Assets | Status |
|----------|-------|-------|-------------|--------|--------|
| **Derive** (fka Lyra) | Derive Chain (OP Stack, ID 957) | CLOB (off-chain orderbook, on-chain settlement) | European | BTC, ETH, +45 assets | Active |
| **Aevo** | Aevo L2 (OP Stack) | CLOB (off-chain orderbook, on-chain settlement) | European | ETH, BTC, altcoins | Active |
| **Premia V3** | Arbitrum One | Concentrated Liquidity AMM + Orderbook | European | WETH, WBTC, ARB, LINK, GMX, + permissionless | Active |
| **Stryke** (fka Dopex) | Arbitrum, Base, Mantle, Sonic, Berachain | CLAMM (built on Uniswap V3 ticks) | American | WETH/USDC, WBTC/USDC | Active |

---

## 1. Derive (formerly Lyra V2)

**Docs:** https://docs.derive.xyz/

### Endpoints

| Environment | REST | WebSocket |
|-------------|------|-----------|
| Production  | `https://api.lyra.finance` (POST JSON-RPC) | `wss://api.lyra.finance/ws` |
| Testnet     | `https://api-demo.lyra.finance` | `wss://api-demo.lyra.finance/ws` |

### Chain Details

- **Derive Chain** — OP Stack L2, Chain ID `957`
- RPC: `https://rpc.lyra.finance`
- Explorer: `https://explorer.lyra.finance`
- Testnet Chain ID: `901`

### Key API Methods

```
public/get_all_instruments    — List all options (filter by instrument_type, currency, expired)
public/get_ticker             — Bid/ask, mark price, greeks, IV for single instrument
public/get_instrument         — Instrument details (strike, expiry, option type)
public/get_latest_signed_feeds — Raw oracle price feeds
public/get_option_settlement_prices — Historical settlement prices
```

**RFQ Flow:**
```
private/send_rfq → private/poll_quotes → private/execute_quote
private/send_quote / private/cancel_quote / private/replace_quote
```

### Ticker Response Fields (Options)

- `option_pricing.delta`, `.gamma`, `.theta`, `.vega`, `.rho`
- `option_pricing.iv`, `.bid_iv`, `.ask_iv`
- `option_pricing.mark_price`, `.forward_price`, `.discount_factor`
- `best_bid_price`, `best_ask_price`, `best_bid_amount`, `best_ask_amount`
- `index_price`, `mark_price`

### Instrument Naming

Same as Deribit: `ETH-28MAR26-3000-C`

### Authentication

Session key based. Register a session key on-chain, then sign requests with it.

**Env vars for SDK:**
- `DERIVE_SESSION_KEY` — Private key for signing
- `DERIVE_WALLET` — LightAccount address
- `DERIVE_SUBACCOUNT_ID` — Target subaccount
- `DERIVE_ENV` — `TEST` or `PROD`

### SDKs

| Language | Package |
|----------|---------|
| Python (recommended) | `pip install derive-client` |
| Python (on-chain signing) | [v2-action-signing-python](https://github.com/derivexyz/v2-action-signing-python) |
| Rust | [cockpit](https://github.com/derivexyz/cockpit) |
| CCXT | Supported |

### Key Contracts (Derive Chain 957)

| Contract | Address |
|----------|---------|
| Sub Accounts | `0xE7603DF191D699d8BD9891b821347dbAb889E5a5` |
| Cash (USDC) | `0x57B03E14d409ADC7fAb6CFc44b5886CAD2D5f02b` |
| SRM (Risk Manager) | `0x28c9ddF9A3B29c2E6a561c1BC520954e5A33de5D` |
| ETH Option Asset | `0x4BB4C3CDc7562f08e9910A0C7D8bB7e108861eB4` |
| ETH Perp Asset | `0xAf65752C4643E25C02F693f9D4FE19cF23a095E3` |
| Vol Feed | `0xb27cb6b08e6c298C8634D73D5F6649665e90d160` |

### Pricing Model

- **CLOB** with Black-Scholes based mark pricing
- Greeks computed server-side, returned via API
- Mark price derived from vol surface, forward price, discount factor
- Pricing data sourced from Block Scholes
- European-style, settle to TWAP of index at expiration

---

## 2. Aevo

**API Docs:** https://api-docs.aevo.xyz/

### Endpoints

| Environment | REST | WebSocket |
|-------------|------|-----------|
| Production  | `https://api.aevo.xyz` | `wss://ws.aevo.xyz` |
| Testnet     | `https://api-testnet.aevo.xyz` | `wss://ws-testnet.aevo.xyz` |

### Key REST Endpoints

```
GET /markets?asset=ETH&instrument_type=OPTION   — All option instruments with greeks/IV
GET /orderbook?instrument_name=ETH-28MAR25-2000-C — Full orderbook
GET /index?asset=ETH                             — Current index (spot) price
GET /expiries?asset=ETH                          — Available expiry dates
GET /assets                                      — All supported assets
GET /instrument/{name}/trade-history             — Trade history
GET /statistics                                  — Exchange stats

POST /orders         — Create order (limit/market)
POST /batch-orders   — Submit multiple orders
DELETE /orders/{id}  — Cancel order
DELETE /orders-all   — Cancel all orders
GET /positions       — Open positions
GET /portfolio       — Portfolio overview
```

### WebSocket Channels

**Public:**
```
orderbook-100ms:{instrument}   — Orderbook snapshots + deltas every 100ms
orderbook-500ms:{instrument}   — Throttled orderbook
trades:{asset}                 — Matched trades stream
trades:{instrument}            — Trades for specific instrument
ticker:{asset}:PERPETUAL       — Ticker updates
index:{asset}                  — Index price updates
```

**Private (after auth):**
```
PUBLISH Create Order / Edit Order / Cancel Order / Cancel All Orders
SUBSCRIBE Orders / Fills / Positions
```

### Orderbook Data Structure

Each level: `[price, amount, implied_volatility]`
- Amount = 0 means level removed
- Checksum provided for integrity verification

### Instrument Naming

`{ASSET}-{DDMMMYY}-{STRIKE}-{C/P}` — e.g., `ETH-28MAR25-2000-C`

### Authentication

1. Generate signing key (random ETH keypair)
2. EIP-712 registration signature (domain: `Aevo Mainnet`, chainId: 1)
3. `POST /register` with account + signing key + signatures

**REST headers:** `AEVO-KEY`, `AEVO-SECRET`, `AEVO-TIMESTAMP`, `AEVO-SIGNATURE` (HMAC-SHA256)

**WebSocket auth:**
```json
{"op": "auth", "data": {"key": "[API Key]", "secret": "[API Secret]"}}
```

### Order Signing (EIP-712)

Every order must be signed. Fields: `maker`, `isBuy`, `limitPrice` (6 decimals), `amount` (6 decimals), `salt`, `instrument` (numeric ID), `timestamp`.

### SDK

**Official Python:** https://github.com/aevoxyz/aevo-sdk

```python
from client import AevoClient
client = AevoClient(signing_key="...", wallet_address="...", api_key="...", api_secret="...", env="mainnet")
markets = client.get_markets("ETH")
```

### Architecture

- Off-chain orderbook (sub-10ms matching) + off-chain risk engine
- On-chain settlement on Aevo L2 (OP Stack, sequencer by Conduit)
- Gasless order placement/cancellation
- European-style, cash-settled options

---

## 3. Premia V3 ("Premia Blue")

**Docs:** https://docs.premia.blue/

### Chain

- **Arbitrum One** (42161) — All pool settlement, trading, collateral
- **Arbitrum Nova** — On-chain orderbook (OrderbookStreamer contract)

### Endpoints

| Type | URL |
|------|-----|
| WebSocket (Mainnet) | `wss://quotes.premia.finance` |
| WebSocket (Testnet) | `wss://test.quotes.premia.finance` |
| Subgraph | `https://api.thegraph.com/subgraphs/name/premian-labs/premia-blue` |
| Containerized REST API | Self-hosted Docker |

### SDK

```bash
yarn add @premia/v3-sdk-public
```

```typescript
import { Premia, SupportedChainId } from '@premia/v3-sdk-public'

const premia = await Premia.initialize({
    provider: `https://arbitrum-mainnet.infura.io/v3/${INFURA_KEY}`,
    chainId: SupportedChainId.ARBITRUM,
    privateKey: PRIVATE_KEY,
    apiKey: PREMIA_API_KEY
})

// Get AMM quote
const quote = await premia.pools.quote(poolAddress, parseEther('1'), true)

// Execute trade
await premia.pools.trade(poolAddress, {
    size: quote.size,
    isBuy: quote.isBuy,
    premiumLimit: quote.approvalAmount,
    referrer: ZeroAddress
})
```

**SDK modules:** `pools`, `options`, `orders`, `analytics`, `pricing`, `vaults`, `tokens`, `pairs`

### WebSocket Protocol

1. Send `AUTH` message with API key
2. Send `FILTER` on **QUOTES** channel (filter by pool, size, side, taker, provider)
3. Send `FILTER` on **RFQ** channel for request-for-quote streams

### Containerized REST API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/pools` | List option pools (filter by base, quote, expiration) |
| `POST` | `/pools` | Deploy new option pool |
| `POST` | `/pool/settle` | Settle expired option |
| `POST` | `/pool/annihilate` | Close position |

### Key Contracts (Arbitrum)

| Contract | Address |
|----------|---------|
| PREMIA token | `0x51fc0f6660482ea73330e414efd7808811a57fa2` |
| VxPremiaProxy | `0x3992690E5405b69d50812470B0250c878bFA9322` |
| Premia Core (Diamond) | `0x89b36CE3491f2258793C7408Bd46aac725973BA2` |
| VolatilitySurfaceOracle | `0x23c74Cb91085c4cB2B76Cea709AE50309f79DBBD` |

### Pricing Model

- **Concentrated Liquidity AMM** — LPs deposit into range orders with user-defined price bounds (Uniswap V3 style, but for options)
- Four liquidity sources aggregated: AMM pools, Orderbook/RFQ, Vaults, External protocols
- `VolatilitySurfaceOracle` provides on-chain IV data
- Taker fee on top of quote price
- European-style, physically settled
- Max expiry: 365 days
- Permissionless pool creation for any ERC-20

### GitHub

- Contracts: https://github.com/Premian-Labs/v3-contracts
- SDK: https://github.com/Premian-Labs/v3-sdk
- Subgraph: https://github.com/Premian-Labs/v3-subgraph

---

## 4. Stryke (formerly Dopex)

**Docs:** https://docs.stryke.xyz/

### Important: SSOVs are deprecated. CLAMM is the active product.

### Chain

- **Arbitrum** (primary) — also on Base, Mantle, Sonic, Berachain

### How CLAMM Works

- Built on top of **Uniswap V3 concentrated liquidity** tick ranges as option strikes
- **American-style options** (exercisable any time before expiry)
- Expiries: **20 minutes to 24 hours**
- Premiums paid in USDC
- 15% protocol fee on purchases, 0% for LPs
- Auto-exercise available (5 min before expiry)

### SDK

**TypeScript SDK:** https://github.com/stryke-xyz/sdk

Modules: `abi/`, `amms/`, `api/`, `chains/`, `handlers/`, `hooks/`, `markets/`, `periphery/`, `tokens/`, `utils/`

### REST APIs (Deprecated)

Located at `docs.stryke.xyz/developers/apis`:
- Trading: Purchase quotes, user positions, exercise
- LP Management: Deposit, positions, withdraw
- Option Markets: Available markets and metadata
- Strikes Chain: Available strikes

### Key Contracts (Arbitrum)

| Contract | Address |
|----------|---------|
| WETH/USDC Option Market | `0x2536974545c28F7C7d17038c7623E8132FbD82bb` |
| WBTC/USDC Option Market | `0xaBa531Ae39Fa20a0D6B16CD1f9b393862aDb602e` |
| Uniswap V3 Handler | `0x6F73aFB6598d7a3881577f884f2E01574aEFC373` |
| PositionManager | `0x467f2E854C53DFC44e592600c4E0E9e86898E84b` |
| ClammRouterV2 | `0x2dD8BF6bf68dD903F32B9dEfB20443305D301fA6` |
| AutoExercise | `0xb223eD797742E096632c39d1b2e0a313750B25FE` |
| SYK Token | `0xACC51FFDeF63fB0c014c882267C3A17261A5eD50` |

### Pricing Model

- Proprietary model replicating volatility smiles
- Premiums computed automatically given strike + quantity
- Short expiries (20min–24hr) create frequent mispricing opportunities

---

## Arb Strategy: CeFi vs DeFi

### 1. Deribit ↔ Derive/Aevo (CLOB vs CLOB)

Both Derive and Aevo use orderbooks like Deribit. Compare:
- Same strike/expiry across venues
- IV differences (bid_iv/ask_iv)
- Account for fees: Deribit taker ~0.03%, Derive/Aevo variable

### 2. Deribit ↔ Premia (CLOB vs AMM)

- Get Deribit mid-market IV
- Get Premia AMM quote for same strike/expiry
- AMM pricing may lag during vol spikes → arb window
- Use `VolatilitySurfaceOracle` on-chain IV vs Deribit IV

### 3. Deribit ↔ Stryke (CLOB vs CLAMM)

- Stryke has ultra-short expiries (20min–24hr) — compare against Deribit 0DTE
- American-style (Stryke) vs European (Deribit) — early exercise premium
- CLAMM pricing model may misprice at strike extremes

### 4. Cross-DeFi (Derive ↔ Aevo ↔ Premia)

- Compare same-instrument pricing across DeFi venues
- AMM (Premia) vs CLOB (Derive/Aevo) — different price discovery speeds
- Arbitrage the vol surface: on-chain oracle IV (Premia) vs CLOB IV (Derive/Aevo)

### Key Considerations

- **Gas costs on Arbitrum** (~$0.01–0.10 per tx) eat into AMM arb profits
- **Derive/Aevo are gasless** for order placement (off-chain orderbook)
- **Liquidity**: Deribit >> Aevo/Derive >> Premia >> Stryke
- **Settlement risk**: DeFi protocols have smart contract risk
- **Capital efficiency**: Need funds on each venue/chain separately
- **Bridge latency**: Cross-chain arb adds bridging time and risk
