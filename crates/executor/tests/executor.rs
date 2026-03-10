use executor::{
    calc_fee_aware_size, execute_atomic_pair, next_retry_delay_ms, ExecutorState, OrderRequest,
    OrderStatus, VenueOrder,
};

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
