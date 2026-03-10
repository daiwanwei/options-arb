use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Partial,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct VenueOrder {
    pub id: String,
    pub venue: String,
    pub status: OrderStatus,
}

impl VenueOrder {
    pub fn new(id: &str, venue: &str) -> Self {
        Self {
            id: id.to_string(),
            venue: venue.to_string(),
            status: OrderStatus::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub venue: String,
    pub instrument: String,
    pub size: f64,
    pub is_buy: bool,
}

impl OrderRequest {
    pub fn new(venue: &str, instrument: &str, size: f64, is_buy: bool) -> Self {
        Self {
            venue: venue.to_string(),
            instrument: instrument.to_string(),
            size,
            is_buy,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AtomicExecutionPlan {
    pub legs: Vec<OrderRequest>,
    pub cancel_on_disconnect: bool,
}

#[derive(Default)]
pub struct ExecutorState {
    orders: HashMap<String, VenueOrder>,
}

impl ExecutorState {
    pub fn track(&mut self, order: VenueOrder) {
        self.orders.insert(order.id.clone(), order);
    }

    pub fn update_status(&mut self, order_id: &str, status: OrderStatus) {
        if let Some(order) = self.orders.get_mut(order_id) {
            order.status = status;
        }
    }

    pub fn get(&self, order_id: &str) -> Option<&VenueOrder> {
        self.orders.get(order_id)
    }
}

pub fn calc_fee_aware_size(capital: f64, price: f64, fee_rate: f64) -> f64 {
    if price <= 0.0 {
        return 0.0;
    }
    capital / (price * (1.0 + fee_rate))
}

pub fn execute_atomic_pair(buy_leg: &OrderRequest, sell_leg: &OrderRequest) -> AtomicExecutionPlan {
    AtomicExecutionPlan {
        legs: vec![buy_leg.clone(), sell_leg.clone()],
        cancel_on_disconnect: true,
    }
}

pub fn next_retry_delay_ms(attempt: u32) -> u64 {
    let value = 500_u64.saturating_mul(2_u64.saturating_pow(attempt));
    value.min(30_000)
}

#[derive(Debug, Clone)]
pub struct PaperFill {
    pub signal_id: String,
    pub entry_price: f64,
    pub exit_price: f64,
    pub size: f64,
}

impl PaperFill {
    pub fn new(signal_id: &str, entry_price: f64, exit_price: f64, size: f64) -> Self {
        Self {
            signal_id: signal_id.to_string(),
            entry_price,
            exit_price,
            size,
        }
    }

    pub fn pnl(&self) -> f64 {
        (self.exit_price - self.entry_price) * self.size
    }
}

#[derive(Debug, Clone)]
pub struct PrometheusSnapshot {
    pub signals_per_min: f64,
    pub fill_rate: f64,
    pub pnl: f64,
    pub avg_latency_ms: f64,
}

#[derive(Default)]
pub struct PaperTrader {
    fills: Vec<PaperFill>,
    signals_seen: u64,
    latencies_ms: Vec<f64>,
}

impl PaperTrader {
    pub fn record_signal_latency_ms(&mut self, latency: f64) {
        self.signals_seen += 1;
        self.latencies_ms.push(latency);
    }

    pub fn record_fill(&mut self, fill: PaperFill) {
        self.signals_seen += 1;
        self.fills.push(fill);
    }

    pub fn total_pnl(&self) -> f64 {
        self.fills.iter().map(PaperFill::pnl).sum()
    }

    pub fn hit_rate(&self) -> f64 {
        if self.fills.is_empty() {
            return 0.0;
        }
        let wins = self.fills.iter().filter(|fill| fill.pnl() > 0.0).count() as f64;
        wins / self.fills.len() as f64
    }

    pub fn metrics_snapshot(&self, minutes: f64) -> PrometheusSnapshot {
        let avg_latency = if self.latencies_ms.is_empty() {
            0.0
        } else {
            self.latencies_ms.iter().sum::<f64>() / self.latencies_ms.len() as f64
        };

        PrometheusSnapshot {
            signals_per_min: if minutes > 0.0 {
                self.signals_seen as f64 / minutes
            } else {
                0.0
            },
            fill_rate: if self.signals_seen > 0 {
                self.fills.len() as f64 / self.signals_seen as f64
            } else {
                0.0
            },
            pnl: self.total_pnl(),
            avg_latency_ms: avg_latency,
        }
    }
}

pub fn check_risk_alert(utilization: f64, max_allowed: f64) -> Option<String> {
    if utilization > max_allowed {
        Some(format!(
            "risk utilization breached: current={utilization:.2}, limit={max_allowed:.2}"
        ))
    } else {
        None
    }
}

pub fn build_grafana_dashboard_template() -> String {
    r#"{
  "title": "options-arb paper trading",
  "panels": [
    {"title": "signals_per_min", "type": "timeseries"},
    {"title": "fill_rate", "type": "timeseries"},
    {"title": "pnl", "type": "timeseries"},
    {"title": "latency_ms", "type": "timeseries"}
  ]
}"#
    .to_string()
}

pub fn render_prometheus(snapshot: &PrometheusSnapshot, risk_utilization: f64) -> String {
    format!(
        "# TYPE options_arb_signals_per_min gauge\noptions_arb_signals_per_min {signals}\n\
# TYPE options_arb_fill_rate gauge\noptions_arb_fill_rate {fill_rate}\n\
# TYPE options_arb_pnl gauge\noptions_arb_pnl {pnl}\n\
# TYPE options_arb_latency_ms gauge\noptions_arb_latency_ms {latency}\n\
# TYPE options_arb_risk_utilization gauge\noptions_arb_risk_utilization {risk}\n",
        signals = snapshot.signals_per_min,
        fill_rate = snapshot.fill_rate,
        pnl = snapshot.pnl,
        latency = snapshot.avg_latency_ms,
        risk = risk_utilization,
    )
}
