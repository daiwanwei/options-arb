use anyhow::Result;
use common::AppConfig;
use executor::{render_prometheus, PaperTrader, PrometheusSnapshot};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = AppConfig::load()?;
    tracing::info!(environment = %cfg.environment, "options-arb boot");

    if let Ok(bind) = std::env::var("METRICS_BIND") {
        run_metrics_server(&bind).await?;
    }

    Ok(())
}

async fn run_metrics_server(bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind).await?;
    tracing::info!(%bind, "metrics endpoint started");

    loop {
        let (mut stream, _addr) = match listener.accept().await {
            Ok(value) => value,
            Err(err) => {
                tracing::warn!(error = %err, "incoming metrics stream error");
                continue;
            }
        };

        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await;
        let request_text = String::from_utf8_lossy(&request);

        let response = build_http_response(&request_text);
        stream.write_all(response.as_bytes()).await?;
    }
}

fn build_http_response(request_text: &str) -> String {
    if request_text.starts_with("GET /metrics") {
        let snapshot = PrometheusSnapshot {
            signals_per_min: 0.0,
            fill_rate: 0.0,
            pnl: PaperTrader::default().total_pnl(),
            avg_latency_ms: 0.0,
        };
        let body = render_prometheus(&snapshot, 0.0);
        return format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
    }

    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::build_http_response;

    #[test]
    fn metrics_path_returns_ok() {
        let response = build_http_response("GET /metrics HTTP/1.1\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 200 OK"));
    }

    #[test]
    fn unknown_path_returns_404() {
        let response = build_http_response("GET /unknown HTTP/1.1\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 404 Not Found"));
    }
}
