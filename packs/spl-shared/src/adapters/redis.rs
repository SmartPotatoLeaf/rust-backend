use crate::config::RedisConfig;
use crate::error::{AppError, Result};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::time::Duration;
use tracing::{info, warn};

/// Type alias for Redis connection pool
pub type RedisPool = Pool<RedisConnectionManager>;

/// Initialize Redis connection pool
pub async fn create_redis_pool(config: &RedisConfig) -> Result<RedisPool> {
    info!("Initializing Redis connection pool...");

    let manager = RedisConnectionManager::new(config.url.as_str())
        .map_err(|e| AppError::Unknown(format!("Failed to create Redis manager: {}", e)))?;

    let pool_size = config.pool_size.unwrap_or(10);
    let connect_timeout = Duration::from_secs(config.connect_timeout.unwrap_or(5));

    let pool = Pool::builder()
        .max_size(pool_size)
        .connection_timeout(connect_timeout)
        .build(manager)
        .await
        .map_err(|e| AppError::Unknown(format!("Failed to create Redis pool: {}", e)))?;

    // Test connection with PING
    let result = check_redis_health(&pool).await;

    if result {
        info!(
            "Redis connection pool initialized successfully with {} connections",
            pool_size
        );
    }

    Ok(pool)
}

/// Health check for Redis connection
pub async fn check_redis_health(pool: &RedisPool) -> bool {
    match pool.get().await {
        Ok(mut conn) => {
            // Try a simple PING command
            let result: std::result::Result<String, redis::RedisError> =
                redis::cmd("PING").query_async(&mut *conn).await;

            match result {
                Ok(response) => response == "PONG",
                Err(e) => {
                    warn!("Redis health check PING failed: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            warn!("Redis health check failed to get connection: {}", e);
            false
        }
    }
}
