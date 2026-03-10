use pricing::{black_scholes_price, detect_surface_arbitrage, put_call_parity_gap, OptionKind, SurfacePoint};

#[test]
fn black_scholes_call_and_put_are_reasonable() {
    let call = black_scholes_price(100.0, 100.0, 0.5, 0.01, 0.2, OptionKind::Call);
    let put = black_scholes_price(100.0, 100.0, 0.5, 0.01, 0.2, OptionKind::Put);

    assert!((call - 5.876).abs() < 0.05);
    assert!((put - 5.377).abs() < 0.05);
}

#[test]
fn put_call_parity_gap_is_near_zero_for_fair_prices() {
    let gap = put_call_parity_gap(5.876, 5.377, 100.0, 100.0, 0.01, 0.5);
    assert!(gap.abs() < 0.05);
}

#[test]
fn surface_arbitrage_detector_flags_calendar_violations() {
    let points = vec![
        SurfacePoint {
            strike: 100.0,
            maturity_years: 0.1,
            iv: 0.35,
        },
        SurfacePoint {
            strike: 100.0,
            maturity_years: 0.3,
            iv: 0.25,
        },
    ];

    let violations = detect_surface_arbitrage(&points);
    assert!(violations.iter().any(|item| item.contains("calendar")));
}
