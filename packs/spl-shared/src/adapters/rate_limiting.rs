use crate::adapters::redis::{check_redis_health, RedisPool};
use crate::error::{AppError, Result};
use redis::AsyncCommands;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Rate limiter service using Redis for distributed rate limiting
#[derive(Clone)]
pub struct RateLimiter {
    pool: RedisPool,
    window_seconds: u64,
    default_limit: u64,
    burst_size: Option<u64>,
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    pub reset_at: u64,
    pub retry_after: Option<u64>,
}

impl RateLimiter {
    /// Create a new rate limiter instance
    pub fn new(
        pool: RedisPool,
        window_seconds: u64,
        default_limit: u64,
        burst_size: Option<u64>,
    ) -> Self {
        Self {
            pool,
            window_seconds,
            default_limit,
            burst_size,
        }
    }

    /// Check if a request is allowed for the given key (e.g., IP address, user ID)
    /// Returns RateLimitInfo with details about the rate limit status
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: Option<u64>,
        window_seconds: Option<u64>,
    ) -> Result<RateLimit> {
        let limit = limit.unwrap_or(self.default_limit);
        let window_seconds = window_seconds.unwrap_or(self.window_seconds);
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = current_time - (current_time % window_seconds);
        let window_end = window_start + window_seconds;
        let redis_key = format!("rate_limit:{}:{}", key, window_start);

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to get Redis connection: {}", e)))?;

        // Use Redis INCR command with expiration for atomic counter
        let count: u64 = conn
            .incr(&redis_key, 1)
            .await
            .map_err(|e| AppError::Unknown(format!("Redis INCR failed: {}", e)))?;

        // Set expiration on first request (count == 1)
        if count == 1 {
            let _: () = conn
                .expire(&redis_key, window_seconds as i64 + 1)
                .await
                .map_err(|e| AppError::Unknown(format!("Redis EXPIRE failed: {}", e)))?;
        }

        let allowed = count <= limit;
        let (remaining, retry_after) = if allowed {
            (limit - count, Some(window_end - current_time))
        } else {
            (0, None)
        };

        debug!(
            key = %key,
            count = count,
            limit = limit,
            allowed = allowed,
            "Rate limit check"
        );

        Ok(RateLimit {
            allowed,
            limit,
            remaining,
            reset_at: window_end,
            retry_after,
        })
    }

    /// Check rate limit with burst capacity
    /// Allows short bursts above the limit if burst_size is configured
    pub async fn check_rate_limit_with_burst(
        &self,
        key: &str,
        limit: Option<u64>,
        window_seconds: Option<u64>,
    ) -> Result<RateLimit> {
        let limit = limit.unwrap_or(self.default_limit);
        let burst_limit = self.burst_size.map(|b| limit + b).unwrap_or(limit);

        let mut info = self
            .check_rate_limit(key, Some(burst_limit), window_seconds)
            .await?;

        // Adjust the reported limit to the actual limit (not burst)
        info.limit = limit;
        info.remaining = info.remaining.min(limit);

        Ok(info)
    }

    /// Reset rate limit for a specific key (useful for testing or admin operations)
    pub async fn reset(&self, key: &str) -> Result<()> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = current_time - (current_time % self.window_seconds);
        let redis_key = format!("rate_limit:{}:{}", key, window_start);

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to get Redis connection: {}", e)))?;

        let _: () = conn
            .del(&redis_key)
            .await
            .map_err(|e| AppError::Unknown(format!("Redis DEL failed: {}", e)))?;

        debug!(key = %key, "Rate limit reset");
        Ok(())
    }

    /// Get current count for a key without incrementing
    pub async fn get_current_count(&self, key: &str) -> Result<u64> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = current_time - (current_time % self.window_seconds);
        let redis_key = format!("rate_limit:{}:{}", key, window_start);

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to get Redis connection: {}", e)))?;

        let count: Option<u64> = conn
            .get(&redis_key)
            .await
            .map_err(|e| AppError::Unknown(format!("Redis GET failed: {}", e)))?;

        Ok(count.unwrap_or(0))
    }

    /// Health check - verify Redis connection is working
    pub async fn health_check(&self) -> bool {
        check_redis_health(&self.pool).await
    }
}
