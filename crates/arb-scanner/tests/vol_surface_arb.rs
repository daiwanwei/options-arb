use arb_scanner::{
    generate_surface_trade_legs, scan_vol_surface_arbitrage, SurfacePointInput,
    SurfaceSignalType,
};

#[test]
fn detects_calendar_and_butterfly_anomalies() {
    let points = vec![
        SurfacePointInput::new("Deribit", 3000.0, 0.05, 0.70),
        SurfacePointInput::new("Deribit", 3000.0, 0.25, 0.50),
        SurfacePointInput::new("Deribit", 2800.0, 0.10, 0.80),
        SurfacePointInput::new("Deribit", 3000.0, 0.10, 0.60),
        SurfacePointInput::new("Deribit", 3200.0, 0.10, 0.85),
    ];

    let signals = scan_vol_surface_arbitrage(&points, 0.08, 0.15);
    assert!(signals.iter().any(|s| s.signal_type == SurfaceSignalType::Calendar));
    assert!(signals.iter().any(|s| s.signal_type == SurfaceSignalType::Butterfly));
}

#[test]
fn generates_multi_leg_trade_plan() {
    let points = vec![
        SurfacePointInput::new("Deribit", 3000.0, 0.05, 0.70),
        SurfacePointInput::new("Premia", 3100.0, 0.05, 0.52),
    ];

    let signals = scan_vol_surface_arbitrage(&points, 0.05, 0.1);
    let legs = generate_surface_trade_legs(&signals[0]);
    assert!(legs.len() >= 2);
}
