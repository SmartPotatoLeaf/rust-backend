use crate::error::{AppError, Result};
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub admin: Option<AdminConfig>,
    pub integrations: IntegrationsConfig,
    pub redis: Option<RedisConfig>,
    pub rate_limiting: Option<RateLimitingConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    /// Allowed CORS origins. When empty or missing, CORS headers are not added.
    pub cors_allowed_origins: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AzureConfig {
    pub connection_string: String,
    pub container_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub connect_timeout: Option<u64>,
    pub idle_timeout: Option<u64>,
    pub max_lifetime: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (e.g., "redis://localhost:6379" or "redis://user:pass@host:6379/0")
    pub url: String,
    /// Connection pool size
    pub pool_size: Option<u32>,
    /// Connection timeout in seconds
    pub connect_timeout: Option<u64>,
    /// Command timeout in seconds
    pub command_timeout: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminConfig {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IntegrationsConfig {
    pub model_serving: ModelServingConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelServingConfig {
    /// Provider type: "tensorflow", "tensorflow_grpc", or "mock"
    pub provider: String,
    /// Base URL for the model serving endpoint
    pub url: String,
    /// Model name to use for predictions
    pub model_name: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Image size (width/height) for the model. Defaults to 256.
    pub image_size: Option<u32>,
    /// Max concurrent requests to the model. Defaults to 10.
    pub concurrency_limit: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    /// Provider type: "azure", "local", or "mock"
    pub provider: String,
    /// Azure connection string (required for azure provider)
    #[serde(skip_serializing)]
    pub connection_string: Option<String>,
    /// Azure container name (required for azure provider)
    pub container_name: Option<String>,
    /// Local filesystem base path (required for local provider)
    pub local_base_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitingConfig {
    /// Enable rate limiting globally
    pub enabled: bool,
    /// Default rate limit for all endpoints (requests per window)
    pub default_limit: u64,
    /// Time window in seconds (e.g., 60 for per-minute limits)
    pub window_seconds: u64,
    /// Burst capacity (allow short bursts)
    pub burst_size: Option<u64>,
    /// Behavior when global rate limiting is disabled: "allow" or "reject"
    /// Default: "allow" (allows requests to continue)
    pub global_behavior: Option<String>,
    /// Behavior when endpoint-specific rate limiting is disabled: "allow" or "reject"
    /// Default: "allow" (allows requests to continue)
    pub endpoint_behavior: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("SPL").separator("__"));

        let config = builder.build().map_err(AppError::ConfigError)?;

        config.try_deserialize().map_err(AppError::ConfigError)
    }
}
