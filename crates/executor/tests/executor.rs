use executor::{
    calc_fee_aware_size, execute_atomic_pair, execute_atomic_pair_live, execute_kill_switch,
    flatten_orders_to_requests, next_retry_delay_ms, ExecutorState, OrderRequest, OrderStatus,
    VenueOrder, VenueOrderClient,
};
use risk_manager::FlattenOrder;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct MockVenueClient {
    seen: Arc<Mutex<Vec<OrderRequest>>>,
}

impl VenueOrderClient for MockVenueClient {
    fn place_order<'a>(
        &'a self,
        request: &'a OrderRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<VenueOrder>> + Send + 'a>> {
        Box::pin(async move {
            self.seen.lock().unwrap().push(request.clone());
            Ok(VenueOrder::new(
                &format!("{}:{}", request.venue, request.instrument),
                &request.venue,
            ))
        })
    }
}

#[test]
fn computes_fee_aware_size() {
    let size = calc_fee_aware_size(1000.0, 200.0, 0.001);
    assert!(size > 4.9 && size < 5.0);
}

#[test]
fn transitions_order_state() {
    let mut state = ExecutorState::default();
    state.track(VenueOrder::new("o1", "Deribit"));
    state.update_status("o1", OrderStatus::Filled);
    assert_eq!(state.get("o1").unwrap().status, OrderStatus::Filled);
}

#[test]
fn executes_atomic_pair_plan() {
    let buy = OrderRequest::new("Deribit", "ETH-28MAR26-3000-C", 1.0, true);
    let sell = OrderRequest::new("Aevo", "ETH-28MAR26-3000-C", 1.0, false);
    let plan = execute_atomic_pair(&buy, &sell);
    assert_eq!(plan.legs.len(), 2);
    assert!(plan.cancel_on_disconnect);
}

#[test]
fn backoff_is_capped() {
    assert_eq!(next_retry_delay_ms(0), 500);
    assert_eq!(next_retry_delay_ms(1), 1000);
    assert_eq!(next_retry_delay_ms(10), 30_000);
}

#[test]
fn maps_flatten_orders_to_actionable_requests() {
    let flatten = vec![
        FlattenOrder {
            instrument: "ETH-28MAR26-3000-C".to_string(),
            size: -2.0,
        },
        FlattenOrder {
            instrument: "BTC-28MAR26-50000-P".to_string(),
            size: 1.5,
        },
    ];

    let requests = flatten_orders_to_requests("Deribit", &flatten);
    assert_eq!(requests.len(), 2);
    assert!(!requests[0].is_buy);
    assert!((requests[0].size - 2.0).abs() < 1e-9);
    assert!(requests[1].is_buy);
    assert!((requests[1].size - 1.5).abs() < 1e-9);
}

#[tokio::test]
async fn executes_atomic_pair_live_via_order_client() {
    let client = MockVenueClient::default();
    let buy = OrderRequest::new("Deribit", "ETH-28MAR26-3000-C", 1.0, true);
    let sell = OrderRequest::new("Aevo", "ETH-28MAR26-3000-C", 1.0, false);

    let orders = execute_atomic_pair_live(&client, &buy, &sell, 2_000)
        .await
        .expect("atomic execution should succeed");

    assert_eq!(orders.len(), 2);
    assert_eq!(client.seen.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn executes_kill_switch_orders_through_executor() {
    let client = MockVenueClient::default();
    let flatten = vec![
        FlattenOrder {
            instrument: "ETH-28MAR26-3000-C".to_string(),
            size: -2.0,
        },
        FlattenOrder {
            instrument: "BTC-28MAR26-50000-P".to_string(),
            size: 1.0,
        },
    ];

    let orders = execute_kill_switch(&client, "Deribit", &flatten, 2_000)
        .await
        .expect("kill switch execution should succeed");

    assert_eq!(orders.len(), 2);
    let seen = client.seen.lock().unwrap();
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].venue, "Deribit");
}
