CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS tickers (
    id BIGSERIAL,
    venue TEXT NOT NULL,
    instrument TEXT NOT NULL,
    bid DOUBLE PRECISION,
    ask DOUBLE PRECISION,
    mid DOUBLE PRECISION,
    iv DOUBLE PRECISION,
    bid_iv DOUBLE PRECISION,
    ask_iv DOUBLE PRECISION,
    greeks JSONB,
    timestamp_ms BIGINT NOT NULL,
    PRIMARY KEY (id, timestamp_ms)
);

CREATE TABLE IF NOT EXISTS orderbook_snapshots (
    id BIGSERIAL PRIMARY KEY,
    venue TEXT NOT NULL,
    instrument TEXT NOT NULL,
    bids JSONB NOT NULL,
    asks JSONB NOT NULL,
    timestamp_ms BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS trades (
    id BIGSERIAL,
    venue TEXT NOT NULL,
    instrument TEXT NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    size DOUBLE PRECISION NOT NULL,
    side TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    PRIMARY KEY (id, timestamp_ms)
);

CREATE TABLE IF NOT EXISTS arb_signals (
    id BIGSERIAL PRIMARY KEY,
    venues TEXT NOT NULL,
    instrument TEXT NOT NULL,
    iv_spread DOUBLE PRECISION NOT NULL,
    estimated_pnl DOUBLE PRECISION NOT NULL,
    signal_type TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS positions (
    id BIGSERIAL PRIMARY KEY,
    venue TEXT NOT NULL,
    instrument TEXT NOT NULL,
    size DOUBLE PRECISION NOT NULL,
    entry_price DOUBLE PRECISION NOT NULL,
    current_pnl DOUBLE PRECISION NOT NULL,
    timestamp_ms BIGINT NOT NULL
);

SELECT create_hypertable('tickers', by_range('timestamp_ms'), if_not_exists => TRUE);
SELECT create_hypertable('trades', by_range('timestamp_ms'), if_not_exists => TRUE);
