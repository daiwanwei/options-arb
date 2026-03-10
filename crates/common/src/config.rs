use serde::Deserialize;

use crate::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub environment: String,
    pub log_level: String,
    pub deribit_ws_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let _ = dotenvy::dotenv();
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("OPTIONS_ARB").separator("__"))
            .build()
            .map_err(|err| AppError::Config(err.to_string()))?;

        settings
            .try_deserialize()
            .map_err(|err| AppError::Config(err.to_string()))
    }
}
