use anyhow::Result;
use common::AppConfig;
use executor::{render_prometheus, PaperTrader, PrometheusSnapshot};
use std::io::{Read, Write};
use std::net::TcpListener;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = AppConfig::load()?;
    tracing::info!(environment = %cfg.environment, "options-arb boot");

    if let Ok(bind) = std::env::var("METRICS_BIND") {
        run_metrics_server(&bind)?;
    }

    Ok(())
}

fn run_metrics_server(bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind)?;
    tracing::info!(%bind, "metrics endpoint started");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!(error = %err, "incoming metrics stream error");
                continue;
            }
        };

        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        let request_text = String::from_utf8_lossy(&request);

        if request_text.starts_with("GET /metrics") {
            let snapshot = PrometheusSnapshot {
                signals_per_min: 0.0,
                fill_rate: 0.0,
                pnl: PaperTrader::default().total_pnl(),
                avg_latency_ms: 0.0,
            };
            let body = render_prometheus(&snapshot, 0.0);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes())?;
        } else {
            stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")?;
        }
    }

    Ok(())
}
