use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_position_per_instrument: f64,
    pub max_position_per_underlying: f64,
    pub max_abs_delta: f64,
    pub max_abs_gamma: f64,
    pub max_abs_vega: f64,
    pub max_margin_utilization: f64,
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub limits: RiskLimits,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub instrument: String,
    pub underlying: String,
    pub size: f64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
}

impl Position {
    pub fn new(
        instrument: &str,
        underlying: &str,
        size: f64,
        delta: f64,
        gamma: f64,
        vega: f64,
    ) -> Self {
        Self {
            instrument: instrument.to_string(),
            underlying: underlying.to_string(),
            size,
            delta,
            gamma,
            vega,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MarginState {
    pub utilization: f64,
}

#[derive(Debug, Clone)]
pub struct TradeIntent {
    pub instrument: String,
    pub underlying: String,
    pub size_delta: f64,
    pub delta_delta: f64,
    pub gamma_delta: f64,
    pub vega_delta: f64,
}

impl TradeIntent {
    pub fn new(
        instrument: &str,
        underlying: &str,
        size_delta: f64,
        delta_delta: f64,
        gamma_delta: f64,
        vega_delta: f64,
    ) -> Self {
        Self {
            instrument: instrument.to_string(),
            underlying: underlying.to_string(),
            size_delta,
            delta_delta,
            gamma_delta,
            vega_delta,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlattenOrder {
    pub instrument: String,
    pub size: f64,
}

pub fn evaluate_pre_trade(
    config: &RiskConfig,
    positions: &[Position],
    margin: &MarginState,
    intent: &TradeIntent,
) -> Result<()> {
    let inst_size: f64 = positions
        .iter()
        .filter(|p| p.instrument == intent.instrument)
        .map(|p| p.size)
        .sum::<f64>()
        + intent.size_delta;
    if inst_size.abs() > config.limits.max_position_per_instrument {
        return Err(anyhow!("instrument position limit breached"));
    }

    let und_size: f64 = positions
        .iter()
        .filter(|p| p.underlying == intent.underlying)
        .map(|p| p.size)
        .sum::<f64>()
        + intent.size_delta;
    if und_size.abs() > config.limits.max_position_per_underlying {
        return Err(anyhow!("underlying position limit breached"));
    }

    let total_delta = positions.iter().map(|p| p.delta).sum::<f64>() + intent.delta_delta;
    let total_gamma = positions.iter().map(|p| p.gamma).sum::<f64>() + intent.gamma_delta;
    let total_vega = positions.iter().map(|p| p.vega).sum::<f64>() + intent.vega_delta;

    if total_delta.abs() > config.limits.max_abs_delta {
        return Err(anyhow!("delta limit breached"));
    }
    if total_gamma.abs() > config.limits.max_abs_gamma {
        return Err(anyhow!("gamma limit breached"));
    }
    if total_vega.abs() > config.limits.max_abs_vega {
        return Err(anyhow!("vega limit breached"));
    }

    if margin.utilization > config.limits.max_margin_utilization {
        return Err(anyhow!("margin utilization limit breached"));
    }

    Ok(())
}

pub fn flatten_orders(positions: &[Position]) -> Vec<FlattenOrder> {
    positions
        .iter()
        .filter(|position| position.size.abs() > 0.0)
        .map(|position| FlattenOrder {
            instrument: position.instrument.clone(),
            size: -position.size,
        })
        .collect()
}
