# Deribit API Reference

Official docs: https://docs.deribit.com/

## Environments

| Environment | HTTP | WebSocket |
|-------------|------|-----------|
| Production  | `https://www.deribit.com/api/v2` | `wss://www.deribit.com/ws/api/v2` |
| Testnet     | `https://test.deribit.com/api/v2` | `wss://test.deribit.com/ws/api/v2` |

## Interfaces

1. **JSON-RPC over WebSocket** — Real-time, bidirectional. Recommended for most use cases.
2. **JSON-RPC over HTTP** — Simple REST-like interface.
3. **FIX API** — Financial Information eXchange protocol for institutional trading.

## Authentication

### Client Credentials Flow

```json
{
  "jsonrpc": "2.0",
  "method": "public/auth",
  "params": {
    "grant_type": "client_credentials",
    "client_id": "<API_KEY>",
    "client_secret": "<API_SECRET>"
  },
  "id": 1
}
```

### Client Signature Flow (More Secure)

Uses HMAC-SHA256 signature with timestamp and nonce. Required params:
- `grant_type`: `"client_signature"`
- `client_id`, `timestamp`, `nonce`, `signature`
- Optional: `data` field for additional entropy

### Response

```json
{
  "access_token": "...",
  "expires_in": 31536000,
  "refresh_token": "...",
  "token_type": "bearer"
}
```

### Other Auth Methods
- `public/exchange_token` — Switch between subaccounts
- `public/fork_token` — Generate tokens for named sessions
- `private/logout` — Graceful WebSocket disconnection (preserves orders when COD enabled)

## Access Scopes

- `account:read`, `account:read_write`
- `trade:read`, `trade:read_write`
- `wallet:read`, `wallet:read_write`
- `block_trade:read`, `block_trade:read_write`
- `block_rfq:read`, `block_rfq:read_write`

Additional: `connection` (default), `session:name`, `mainaccount`, `expires:NUMBER`, `ip:ADDR`

## Instrument Naming Convention

Options: `{UNDERLYING}-{EXPIRY}-{STRIKE}-{TYPE}`

Examples:
- `BTC-27DEC24-50000-C` — BTC Call, strike 50000, expiry Dec 27 2024
- `ETH-28MAR25-3000-P` — ETH Put, strike 3000, expiry Mar 28 2025
- `BTC-PERPETUAL` — BTC perpetual futures
- `BTC-28MAR25` — BTC futures expiring Mar 28 2025

## WebSocket Message Format (JSON-RPC 2.0)

### Request

```json
{
  "jsonrpc": "2.0",
  "method": "public/subscribe",
  "params": {
    "channels": ["book.BTC-PERPETUAL.100ms"]
  },
  "id": 1
}
```

### Notification (pushed by server)

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "trades.BTC-PERPETUAL.raw",
    "data": [...]
  }
}
```

## WebSocket Subscription Channels

### Public Market Data Channels

| Channel Pattern | Description |
|-----------------|-------------|
| `book.{instrument}.{interval}` | Order book updates (raw, 100ms, agg2) |
| `ticker.{instrument}.{interval}` | Ticker data (raw, 100ms) |
| `trades.{instrument}.{interval}` | Trade execution data (raw, 100ms) |
| `trades.{kind}.{currency}.{interval}` | Trades by kind (option/future) and currency |
| `quote.{instrument}` | Best bid/ask quote |
| `incremental_ticker.{instrument}` | Incremental ticker updates |
| `markprice.options.{index_name}` | Options mark prices |
| `perpetual.{instrument}.{interval}` | Perpetual contract data (funding rate) |
| `instrument.state.{kind}.{currency}` | Instrument state changes |
| `platform_state` | Platform status |
| `deribit_price_index.{index}` | Price index (btc_usd, eth_usd, etc.) |
| `deribit_volatility_index.{index}` | DVOL volatility index |
| `estimated_expiration_price.{index}` | Estimated expiration prices |

### Private User Channels (require auth)

| Channel Pattern | Description |
|-----------------|-------------|
| `user.orders.{instrument}.{interval}` | Order updates |
| `user.orders.{kind}.{currency}.{interval}` | Orders by kind/currency |
| `user.trades.{instrument}.{interval}` | User trade fills |
| `user.trades.{kind}.{currency}.{interval}` | User trades by kind/currency |
| `user.changes.{instrument}.{interval}` | Combined order/trade/position changes |
| `user.changes.{kind}.{currency}.{interval}` | Changes by kind/currency |
| `user.portfolio.{currency}` | Portfolio/margin updates |
| `user.mmp_trigger.{index_name}` | Market maker protection triggers |

### Interval Options
- `raw` — Every event, no aggregation
- `100ms` — Aggregated every 100ms
- `agg2` — Further aggregated (book only)

### Subscription Limits
- Max **500 channels** per WebSocket connection
- Batch subscriptions in a single request for efficiency

## Key Market Data Endpoints (REST)

### Instruments
- `public/get_instruments` — List all instruments by currency and kind
  - Params: `currency` (BTC, ETH, SOL, etc.), `kind` (option, future, spot), `expired` (bool)
- `public/get_contract_size` — Contract multiplier for instrument
- `public/get_currencies` — List all supported currencies

### Order Book & Pricing
- `public/get_book_summary_by_currency` — Market stats for all instruments in a currency
- `public/get_book_summary_by_instrument` — Market stats for single instrument
- `public/get_order_book` — Full order book for instrument
- `public/ticker` — Current ticker data

### Historical Data
- `public/get_last_trades_by_instrument` — Recent trades
- `public/get_last_trades_by_currency` — Recent trades by currency
- `public/get_last_settlements_by_instrument` — Settlement history
- `public/get_delivery_prices` — Historical settlement prices for indices
- `public/get_funding_rate_history` — Historical funding rates
- `public/get_tradingview_chart_data` — OHLCV candle data

### Volatility & Index
- `public/get_index_price` — Current index price
- `public/get_volatility_index_data` — DVOL historical data

## Trading Endpoints

### Order Placement
- `private/buy` — Place buy order
- `private/sell` — Place sell order
- Params: `instrument_name`, `amount`, `type` (limit/market/stop_limit/stop_market), `price`, `label`

### Order Management
- `private/edit` — Modify existing order
- `private/cancel` — Cancel by order ID
- `private/cancel_by_label` — Cancel by label
- `private/cancel_all` — Cancel all open orders
- `private/cancel_all_by_instrument` — Cancel all for instrument
- `private/cancel_all_by_currency` — Cancel all for currency
- `private/cancel_all_by_kind_or_type` — Cancel by kind/type
- `private/close_position` — Close entire position

### Order Queries
- `private/get_open_orders_by_instrument` — Open orders for instrument
- `private/get_open_orders_by_currency` — Open orders for currency
- `private/get_order_state` — Single order status
- `private/get_order_history_by_instrument` — Order history
- `private/get_order_history_by_currency` — Order history by currency
- `private/get_user_trades_by_instrument` — User trade history
- `private/get_user_trades_by_currency` — User trade history by currency
- `private/get_settlement_history_by_instrument` — Settlement history

### Market Maker Protection (MMP)
- `private/set_mmp_config` — Configure MMP parameters
- `private/get_mmp_config` — Get current MMP config
- `private/reset_mmp` — Reset MMP after trigger

## Account & Position Endpoints

- `private/get_account_summary` — Balance, equity, margin details per currency
- `private/get_account_summaries` — All currency summaries
- `private/get_positions` — All open positions with P&L
- `private/get_position` — Single instrument position
- `private/get_margins` — Margin calculation for an order
- `private/simulate_portfolio` — Simulate margin for positions (rate limited: 1 req/sec)

## Session Management

- `public/set_heartbeat` — Enable heartbeat with interval
- `public/disable_heartbeat` — Disable heartbeat
- `private/enable_cancel_on_disconnect` — Enable COD (cancel orders on disconnect)
- `private/disable_cancel_on_disconnect` — Disable COD
- `private/get_cancel_on_disconnect` — Check COD status

## Rate Limits

- Standard API rate limits apply per endpoint
- `get_transaction_log`: 1 req/sec
- `simulate_portfolio`: 1 req/sec
- Order-to-Volume ratio threshold: ~10,000 BTC changes per 1 BTC volume
- WebSocket: max 500 channels per connection
- Excessive errors may result in IP banning

## Best Practices

1. **Use WebSocket for real-time data** — Prefer `raw` interval subscriptions over REST polling
2. **Batch subscriptions** — Subscribe to multiple channels in one request
3. **Use `100ms` interval** for most use cases — `raw` only if you need every tick
4. **Validate order book** with `prev_change_id` sequence numbers
5. **Implement reconnection logic** with exponential backoff
6. **Use testnet first** for development and testing
7. **Store credentials securely** — Client secrets display only once
8. **Use client_signature auth** for production (more secure than client_credentials)

## Combo Books (Multi-Leg Strategies)

- `private/create_combo` — Create or find existing combo
- `private/get_leg_prices` — Calculate individual leg prices
- `public/get_combo_details` — Combo information
- `public/get_combo_ids` — List available combos
- `public/get_combos` — Active combos by currency

## Block Trading

- `private/verify_block_trade` — Step 1: Generate signature
- `private/execute_block_trade` — Step 2: Execute with counterparty
- `private/simulate_block_trade` — Validate without executing
- `private/get_block_trade` — Get trade details
- `private/get_block_trades` — List user's block trades

## Block RFQ (Request for Quote)

- `private/create_block_rfq` — Create RFQ
- `private/accept_block_rfq` — Accept quote
- `private/add_block_rfq_quote` — Maker adds quote
- `private/get_block_rfqs` — List RFQs
- `private/get_block_rfq_quotes` — Get open quotes
