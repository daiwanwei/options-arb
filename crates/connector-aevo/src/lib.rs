use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

pub const AEVO_REST_BASE: &str = "https://api.aevo.xyz";
pub const AEVO_WS_BASE: &str = "wss://ws.aevo.xyz";

pub fn build_markets_url(asset: &str, instrument_type: &str) -> String {
    format!(
        "{AEVO_REST_BASE}/markets?asset={asset}&instrument_type={instrument_type}"
    )
}

pub fn orderbook_channel(instrument: &str) -> String {
    format!("orderbook-100ms:{instrument}")
}

pub fn trades_channel(asset: &str) -> String {
    format!("trades:{asset}")
}

#[derive(Debug, Clone)]
pub struct AevoLocalOrderBook {
    pub bids: BTreeMap<i64, f64>,
    pub asks: BTreeMap<i64, f64>,
}

impl AevoLocalOrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn apply_snapshot(&mut self, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) {
        self.bids.clear();
        self.asks.clear();
        update_side(&mut self.bids, bids);
        update_side(&mut self.asks, asks);
    }

    pub fn apply_delta(
        &mut self,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
        expected_checksum: u64,
    ) -> Result<()> {
        update_side(&mut self.bids, bids);
        update_side(&mut self.asks, asks);

        let current = compute_orderbook_checksum(self);
        if current != expected_checksum {
            return Err(anyhow!(
                "checksum mismatch: expected {expected_checksum}, got {current}"
            ));
        }

        Ok(())
    }
}

pub fn compute_orderbook_checksum(book: &AevoLocalOrderBook) -> u64 {
    let mut value = 0_u64;

    for (price, size) in book.bids.iter().take(5) {
        value = value.wrapping_add(*price as u64).wrapping_add(*size as u64);
    }
    for (price, size) in book.asks.iter().take(5) {
        value = value.wrapping_add(*price as u64).wrapping_add(*size as u64);
    }

    value
}

fn update_side(side: &mut BTreeMap<i64, f64>, levels: Vec<(f64, f64)>) {
    for (price, size) in levels {
        let key = (price * 10000.0).round() as i64;
        if size <= 0.0 {
            side.remove(&key);
        } else {
            side.insert(key, size);
        }
    }
}
