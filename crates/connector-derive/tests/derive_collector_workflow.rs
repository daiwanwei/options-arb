use connector_derive::{
    build_get_all_instruments_request, build_get_ticker_request, build_session_key_auth_request,
    parse_instrument_symbols,
};

#[test]
fn builds_get_all_instruments_request() {
    let req = build_get_all_instruments_request(7, "ETH");
    assert_eq!(req["method"], "public/get_all_instruments");
    assert_eq!(req["params"]["base_currency"], "ETH");
}

#[test]
fn builds_session_key_auth_request() {
    let req = build_session_key_auth_request(9, "session-key");
    assert_eq!(req["method"], "private/login");
    assert_eq!(req["params"]["grant_type"], "session_key");
}

#[test]
fn parses_instrument_names_from_jsonrpc_result() {
    let sample = serde_json::json!({
        "result": [
            {"instrument_name": "ETH-28MAR26-3000-C"},
            {"instrument_name": "ETH-28MAR26-3000-P"}
        ]
    });

    let names = parse_instrument_symbols(&sample);
    assert_eq!(names.len(), 2);
}

#[test]
fn builds_get_ticker_request() {
    let req = build_get_ticker_request(10, "ETH-28MAR26-3000-C");
    assert_eq!(req["method"], "public/get_ticker");
    assert_eq!(req["params"]["instrument_name"], "ETH-28MAR26-3000-C");
}
