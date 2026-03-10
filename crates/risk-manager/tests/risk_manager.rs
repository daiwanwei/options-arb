use risk_manager::{
    evaluate_pre_trade, flatten_orders, MarginState, Position, RiskConfig, RiskLimits, TradeIntent,
};

#[test]
fn rejects_trade_when_position_limit_breached() {
    let config = RiskConfig {
        limits: RiskLimits {
            max_position_per_instrument: 10.0,
            max_position_per_underlying: 30.0,
            max_abs_delta: 50.0,
            max_abs_gamma: 10.0,
            max_abs_vega: 200.0,
            max_margin_utilization: 0.8,
        },
    };

    let positions = vec![Position::new(
        "ETH-28MAR26-3000-C",
        "ETH",
        9.0,
        10.0,
        1.0,
        20.0,
    )];
    let margin = MarginState { utilization: 0.3 };
    let intent = TradeIntent::new("ETH-28MAR26-3000-C", "ETH", 3.0, 2.0, 0.2, 10.0);

    let result = evaluate_pre_trade(&config, &positions, &margin, &intent);
    assert!(result.is_err());
}

#[test]
fn creates_flatten_orders_for_kill_switch() {
    let positions = vec![
        Position::new("ETH-28MAR26-3000-C", "ETH", 2.0, 0.0, 0.0, 0.0),
        Position::new("BTC-27DEC24-50000-P", "BTC", -1.0, 0.0, 0.0, 0.0),
    ];

    let orders = flatten_orders(&positions);
    assert_eq!(orders.len(), 2);
    assert_eq!(orders[0].size, -2.0);
    assert_eq!(orders[1].size, 1.0);
}
