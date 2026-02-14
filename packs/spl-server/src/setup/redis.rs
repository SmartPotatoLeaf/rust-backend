use spl_shared::adapters::redis::{create_redis_pool, RedisPool};
use spl_shared::config::RedisConfig;
use std::sync::Arc;
use tracing::{error, info};

pub async fn initialize_redis(config: &Option<RedisConfig>) -> Option<Arc<RedisPool>> {
    if config.is_none() {
        info!("Redis configuration not found, Redis will not be initialized.");
        return None;
    }

    let redis_config = config.as_ref().unwrap();
    info!("Initializing Redis connection...");

    match create_redis_pool(redis_config).await {
        Ok(pool) => {
            info!("Redis connection pool initialized successfully");
            Some(Arc::new(pool))
        }
        Err(e) => {
            error!("Failed to create Redis connection pool: {}", e);
            info!("Redis will not be available");
            None
        }
    }
}
