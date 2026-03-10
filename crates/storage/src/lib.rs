use anyhow::Result;
use common::types::{Ticker, Trade};
use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub raw_ticker_retention_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            raw_ticker_retention_days: 30,
        }
    }
}

pub fn migrations_sql() -> &'static str {
    r#"
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS tickers (
    id BIGSERIAL PRIMARY KEY,
    venue TEXT NOT NULL,
    instrument TEXT NOT NULL,
    bid DOUBLE PRECISION,
    ask DOUBLE PRECISION,
    mid DOUBLE PRECISION,
    iv DOUBLE PRECISION,
    bid_iv DOUBLE PRECISION,
    ask_iv DOUBLE PRECISION,
    greeks JSONB,
    timestamp_ms BIGINT NOT NULL
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
    id BIGSERIAL PRIMARY KEY,
    venue TEXT NOT NULL,
    instrument TEXT NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    size DOUBLE PRECISION NOT NULL,
    side TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL
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
"#
}

pub fn retention_policy_sql(config: &StorageConfig) -> String {
    format!(
        "SELECT add_retention_policy('tickers', INTERVAL '{} days', if_not_exists => TRUE);",
        config.raw_ticker_retention_days
    )
}

#[derive(Clone)]
pub struct SqlStorage {
    pool: Pool<Postgres>,
}

impl SqlStorage {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = Pool::<Postgres>::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self, config: &StorageConfig) -> Result<()> {
        for statement in migrations_sql()
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            sqlx::query(&format!("{statement};")).execute(&self.pool).await?;
        }

        sqlx::query(&retention_policy_sql(config))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_ticker(&self, ticker: &Ticker) -> Result<()> {
        let greeks = serde_json::to_value(&ticker.greeks)?;
        sqlx::query(
            "INSERT INTO tickers (venue, instrument, bid, ask, mid, iv, bid_iv, ask_iv, greeks, timestamp_ms)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
        )
        .bind(format!("{:?}", ticker.venue))
        .bind(&ticker.instrument.venue_symbol)
        .bind(ticker.bid)
        .bind(ticker.ask)
        .bind(ticker.mid)
        .bind(ticker.iv)
        .bind(ticker.bid_iv)
        .bind(ticker.ask_iv)
        .bind(greeks)
        .bind(ticker.timestamp_ms)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_trade(&self, trade: &Trade) -> Result<()> {
        sqlx::query(
            "INSERT INTO trades (venue, instrument, price, size, side, timestamp_ms)
             VALUES ($1,$2,$3,$4,$5,$6)",
        )
        .bind(format!("{:?}", trade.venue))
        .bind(&trade.instrument.venue_symbol)
        .bind(trade.price)
        .bind(trade.size)
        .bind(format!("{:?}", trade.side))
        .bind(trade.timestamp_ms)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
