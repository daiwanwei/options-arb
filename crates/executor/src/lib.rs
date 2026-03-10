use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use anyhow::{anyhow, Result};
use futures::future::join_all;
use risk_manager::FlattenOrder;
use tokio::time::{timeout, Duration};

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

pub trait VenueOrderClient {
    fn place_order<'a>(
        &'a self,
        request: &'a OrderRequest,
    ) -> Pin<Box<dyn Future<Output = Result<VenueOrder>> + Send + 'a>>;
}

pub async fn execute_atomic_pair_live<C: VenueOrderClient + Sync>(
    client: &C,
    buy_leg: &OrderRequest,
    sell_leg: &OrderRequest,
    timeout_ms: u64,
) -> Result<Vec<VenueOrder>> {
    let execution = timeout(
        Duration::from_millis(timeout_ms),
        async {
            let buy_future = client.place_order(buy_leg);
            let sell_future = client.place_order(sell_leg);
            tokio::try_join!(buy_future, sell_future)
        },
    )
    .await
    .map_err(|_| anyhow!("atomic execution timed out"))?;

    let (buy_order, sell_order) = execution?;
    Ok(vec![buy_order, sell_order])
}

pub fn flatten_orders_to_requests(venue: &str, flatten_orders: &[FlattenOrder]) -> Vec<OrderRequest> {
    flatten_orders
        .iter()
        .filter(|order| order.size.abs() > 0.0)
        .map(|order| OrderRequest {
            venue: venue.to_string(),
            instrument: order.instrument.clone(),
            size: order.size.abs(),
            is_buy: order.size > 0.0,
        })
        .collect()
}

pub async fn execute_kill_switch<C: VenueOrderClient + Sync>(
    client: &C,
    venue: &str,
    flatten_orders: &[FlattenOrder],
    timeout_ms: u64,
) -> Result<Vec<Result<VenueOrder>>> {
    let requests = flatten_orders_to_requests(venue, flatten_orders);
    let timeout_duration = Duration::from_millis(timeout_ms);

    let tasks: Vec<_> = requests
        .into_iter()
        .map(|request| {
            let client_ref = client;
            async move {
                timeout(timeout_duration, client_ref.place_order(&request))
                    .await
                    .map_err(|_| anyhow!("kill switch order timed out"))?
            }
        })
        .collect();

    Ok(join_all(tasks).await)
}

#[derive(Debug, Clone, Default)]
pub struct VenueAuthConfig {
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub auth_header: Option<String>,
}

#[derive(Clone, Default)]
pub struct HttpOrderClient {
    client: reqwest::Client,
    venue_endpoints: HashMap<String, String>,
    venue_auth: HashMap<String, VenueAuthConfig>,
}

#[derive(Debug, serde::Serialize)]
struct HttpOrderPayload<'a> {
    instrument: &'a str,
    size: f64,
    side: &'a str,
}

#[derive(Debug, serde::Deserialize)]
struct HttpOrderResponse {
    order_id: Option<String>,
    status: Option<String>,
}

impl HttpOrderClient {
    pub fn with_endpoint(mut self, venue: &str, endpoint: &str) -> Self {
        self.venue_endpoints
            .insert(venue.to_string(), endpoint.to_string());
        self
    }

    pub fn with_endpoint_and_auth(
        mut self,
        venue: &str,
        endpoint: &str,
        auth_header: Option<String>,
    ) -> Self {
        self.venue_endpoints
            .insert(venue.to_string(), endpoint.to_string());

        if let Some(value) = auth_header {
            self.venue_auth
                .entry(venue.to_string())
                .or_default()
                .auth_header = Some(value);
        }

        self
    }

    pub fn with_auth_config(mut self, venue: &str, auth: VenueAuthConfig) -> Self {
        self.venue_auth.insert(venue.to_string(), auth);
        self
    }
}

fn auth_headers_for_venue(venue: &str, auth: Option<&VenueAuthConfig>) -> Vec<(&'static str, String)> {
    let Some(auth) = auth else {
        return Vec::new();
    };

    let mut headers = Vec::new();

    if let Some(value) = auth.auth_header.as_ref() {
        headers.push(("Authorization", value.clone()));
    }

    if venue.eq_ignore_ascii_case("aevo") {
        if let Some(key) = auth.api_key.as_ref() {
            headers.push(("AEVO-KEY", key.clone()));
        }
        if let Some(secret) = auth.api_secret.as_ref() {
            headers.push(("AEVO-SECRET", secret.clone()));
        }
    }

    headers
}

impl VenueOrderClient for HttpOrderClient {
    fn place_order<'a>(
        &'a self,
        request: &'a OrderRequest,
    ) -> Pin<Box<dyn Future<Output = Result<VenueOrder>> + Send + 'a>> {
        Box::pin(async move {
            let endpoint = self
                .venue_endpoints
                .get(&request.venue)
                .ok_or_else(|| anyhow!("missing endpoint for venue {}", request.venue))?;

            let payload = HttpOrderPayload {
                instrument: &request.instrument,
                size: request.size,
                side: if request.is_buy { "buy" } else { "sell" },
            };

            let mut request_builder = self.client.post(endpoint).json(&payload);
            for (header_name, header_value) in
                auth_headers_for_venue(&request.venue, self.venue_auth.get(&request.venue))
            {
                request_builder = request_builder.header(header_name, header_value);
            }

            let response = request_builder.send().await?.error_for_status()?;

            let parsed = response.json::<HttpOrderResponse>().await.ok();
            let order_id = parsed
                .as_ref()
                .and_then(|item| item.order_id.clone())
                .unwrap_or_else(|| format!("{}:{}", request.venue, request.instrument));
            let status = match parsed
                .as_ref()
                .and_then(|item| item.status.as_deref())
                .unwrap_or("pending")
            {
                "filled" => OrderStatus::Filled,
                "partial" => OrderStatus::Partial,
                "cancelled" => OrderStatus::Cancelled,
                _ => OrderStatus::Pending,
            };

            Ok(VenueOrder {
                id: order_id,
                venue: request.venue.clone(),
                status,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{auth_headers_for_venue, VenueAuthConfig};

    #[test]
    fn maps_aevo_auth_config_to_expected_headers() {
        let auth = VenueAuthConfig {
            api_key: Some("test-key".to_string()),
            api_secret: Some("test-secret".to_string()),
            auth_header: Some("Bearer test-token".to_string()),
        };

        let headers = auth_headers_for_venue("Aevo", Some(&auth));
        assert!(headers.iter().any(|(k, v)| *k == "Authorization" && v == "Bearer test-token"));
        assert!(headers.iter().any(|(k, v)| *k == "AEVO-KEY" && v == "test-key"));
        assert!(headers.iter().any(|(k, v)| *k == "AEVO-SECRET" && v == "test-secret"));
    }

    #[test]
    fn skips_aevo_specific_headers_for_other_venues() {
        let auth = VenueAuthConfig {
            api_key: Some("test-key".to_string()),
            api_secret: Some("test-secret".to_string()),
            auth_header: Some("Bearer deribit-token".to_string()),
        };

        let headers = auth_headers_for_venue("Deribit", Some(&auth));
        assert!(headers.iter().any(|(k, v)| *k == "Authorization" && v == "Bearer deribit-token"));
        assert!(!headers.iter().any(|(k, _)| *k == "AEVO-KEY"));
        assert!(!headers.iter().any(|(k, _)| *k == "AEVO-SECRET"));
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
