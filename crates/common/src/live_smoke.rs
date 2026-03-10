use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SmokeResult {
    pub venue: String,
    pub phase: String,
    pub ok: bool,
    pub details: String,
}

pub fn run_live_smoke() -> Result<Vec<SmokeResult>, String> {
    if std::env::var("RUN_LIVE_SMOKE").unwrap_or_default() != "1" {
        return Ok(Vec::new());
    }

    if std::env::var("LIVE_SMOKE_DRY_RUN").unwrap_or_default() == "1" {
        return Ok(vec![
            SmokeResult {
                venue: "deribit".to_string(),
                phase: "endpoint".to_string(),
                ok: true,
                details: "dry-run".to_string(),
            },
            SmokeResult {
                venue: "derive".to_string(),
                phase: "endpoint".to_string(),
                ok: true,
                details: "dry-run".to_string(),
            },
            SmokeResult {
                venue: "aevo".to_string(),
                phase: "endpoint".to_string(),
                ok: true,
                details: "dry-run".to_string(),
            },
            SmokeResult {
                venue: "premia".to_string(),
                phase: "endpoint".to_string(),
                ok: true,
                details: "dry-run".to_string(),
            },
            SmokeResult {
                venue: "stryke".to_string(),
                phase: "endpoint".to_string(),
                ok: true,
                details: "dry-run".to_string(),
            },
        ]);
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|err| err.to_string())?;

    let mut out = Vec::new();
    out.push(check_get_with_retry(
        &client,
        "deribit",
        "https://www.deribit.com/api/v2/public/get_time",
        "result",
    ));
    out.push(check_post_with_retry(
        &client,
        "derive",
        "https://api.lyra.finance",
        serde_json::json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"public/get_all_instruments",
            "params":{"base_currency":"ETH"}
        }),
        "result",
    ));
    out.push(check_get_with_retry(
        &client,
        "aevo",
        "https://api.aevo.xyz/markets?asset=ETH&instrument_type=OPTION",
        "[",
    ));
    out.push(check_post_with_retry(
        &client,
        "premia",
        "https://api.thegraph.com/subgraphs/name/premian-labs/premia-blue",
        serde_json::json!({"query":"{ _meta { hasIndexingErrors } }"}),
        "data",
    ));
    out.push(check_get_with_retry(
        &client,
        "stryke",
        "https://docs.stryke.xyz/",
        "<html",
    ));

    Ok(out)
}

fn check_get_with_retry(
    client: &reqwest::blocking::Client,
    venue: &str,
    url: &str,
    expected_fragment: &str,
) -> SmokeResult {
    for attempt in 1..=3 {
        let response = client.get(url).send();
        match response {
            Ok(value) => {
                let status = value.status();
                let body = value.text().unwrap_or_default();
                if status.is_success() && body.contains(expected_fragment) {
                    return SmokeResult {
                        venue: venue.to_string(),
                        phase: "response-shape".to_string(),
                        ok: true,
                        details: format!("ok on attempt {attempt}"),
                    };
                }
            }
            Err(_err) => {}
        }
    }

    SmokeResult {
        venue: venue.to_string(),
        phase: "response-shape".to_string(),
        ok: false,
        details: "failed after 3 attempts".to_string(),
    }
}

fn check_post_with_retry(
    client: &reqwest::blocking::Client,
    venue: &str,
    url: &str,
    payload: serde_json::Value,
    expected_fragment: &str,
) -> SmokeResult {
    for attempt in 1..=3 {
        let response = client.post(url).json(&payload).send();
        match response {
            Ok(value) => {
                let status = value.status();
                let body = value.text().unwrap_or_default();
                if status.is_success() && body.contains(expected_fragment) {
                    return SmokeResult {
                        venue: venue.to_string(),
                        phase: "response-shape".to_string(),
                        ok: true,
                        details: format!("ok on attempt {attempt}"),
                    };
                }
            }
            Err(_err) => {}
        }
    }

    SmokeResult {
        venue: venue.to_string(),
        phase: "response-shape".to_string(),
        ok: false,
        details: "failed after 3 attempts".to_string(),
    }
}
