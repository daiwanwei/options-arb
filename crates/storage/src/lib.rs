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
    include_str!("../migrations/001_init_timescaledb.sql")
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
        sqlx::migrate!("./migrations").run(&self.pool).await?;

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
