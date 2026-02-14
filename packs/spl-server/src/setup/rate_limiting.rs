use spl_shared::adapters::rate_limiting::RateLimiter;
use spl_shared::adapters::redis::RedisPool;
use spl_shared::config::AppConfig;
use spl_shared::http::middleware::rate_limit::RateLimitState;
use std::sync::Arc;
use tracing::{info, warn};

pub fn initialize_rate_limiting(
    config: &AppConfig,
    redis_pool: Option<Arc<RedisPool>>,
) -> Arc<RateLimitState> {
    let rt_config = &config.rate_limiting;
    let disabled = Arc::new(RateLimitState::disabled());

    if rt_config.is_none() {
        info!("Rate limiting configuration not found, rate limiting will be disabled.");
        return disabled;
    }

    let rt_config = rt_config.as_ref().unwrap();

    if !rt_config.enabled {
        info!("Rate limiting is disabled in configuration");
        return disabled;
    }

    if redis_pool.is_none() {
        warn!("Rate limiting is enabled but Redis pool is not available. Rate limiting will be disabled.");
        return disabled;
    }

    let pool = redis_pool.unwrap();
    info!("Initializing rate limiting with Redis...");

    let limiter = RateLimiter::new(
        (*pool).clone(),
        rt_config.window_seconds,
        rt_config.default_limit,
        rt_config.burst_size,
    );

    let global_behavior = rt_config
        .global_behavior
        .as_deref()
        .unwrap_or("allow")
        .into();

    let endpoint_behavior = rt_config
        .endpoint_behavior
        .as_deref()
        .unwrap_or("allow")
        .into();

    info!("Rate limiting initialized successfully");

    Arc::new(RateLimitState::new(
        limiter,
        rt_config.enabled,
        rt_config.window_seconds,
        global_behavior,
        endpoint_behavior,
    ))
}
