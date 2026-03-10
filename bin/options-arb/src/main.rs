use anyhow::Result;
use common::AppConfig;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = AppConfig::load()?;
    tracing::info!(environment = %cfg.environment, "options-arb boot");
    Ok(())
}
