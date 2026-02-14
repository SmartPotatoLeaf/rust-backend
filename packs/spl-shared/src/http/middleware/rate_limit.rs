use crate::adapters::rate_limiting::{RateLimit, RateLimiter};
use crate::http::responses::StatusResponse;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, warn};

/// Middleware state for rate limiting
#[derive(Clone)]
pub struct RateLimitState {
    pub limiter: Option<Arc<RateLimiter>>,
    pub enabled: bool,
    pub window_seconds: u64,
    pub global_behavior: RateLimitBehavior,
    pub endpoint_behavior: RateLimitBehavior,
}

impl<S> From<S> for RateLimitBehavior
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        match value.as_ref().to_lowercase().as_str() {
            "reject" => Self::RejectWhenDisabled,
            _ => Self::AllowWhenDisabled, // Default to allow
        }
    }
}

impl RateLimitState {
    pub fn new(
        limiter: RateLimiter,
        enabled: bool,
        window_seconds: u64,
        global_behavior: RateLimitBehavior,
        endpoint_behavior: RateLimitBehavior,
    ) -> Self {
        Self {
            limiter: Some(Arc::new(limiter)),
            enabled,
            window_seconds,
            global_behavior,
            endpoint_behavior,
        }
    }

    /// Create a disabled rate limiter (no-op)
    pub fn disabled() -> Self {
        Self {
            limiter: None,
            enabled: false,
            window_seconds: 60, // Default, won't be used
            global_behavior: RateLimitBehavior::default(),
            endpoint_behavior: RateLimitBehavior::default(),
        }
    }
}

/// Behavior when rate limiting is disabled
#[derive(Clone, Debug, Copy)]
pub enum RateLimitBehavior {
    /// Allow the request to continue (default)
    AllowWhenDisabled,
    /// Reject the request with 503 Service Unavailable
    RejectWhenDisabled,
}

impl Default for RateLimitBehavior {
    fn default() -> Self {
        Self::AllowWhenDisabled
    }
}

/// Rate limiting configuration for a specific endpoint
#[derive(Clone, Debug)]
pub struct EndpointRateLimit {
    /// Requests per window
    pub limit: u64,
    /// Use burst capacity
    pub use_burst: bool,
    /// Behavior when rate limiting is disabled
    pub behavior: RateLimitBehavior,
    /// Custom time window in seconds (if None, uses global window_seconds)
    pub window_seconds: Option<u64>,
}

impl EndpointRateLimit {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            use_burst: false,
            behavior: RateLimitBehavior::default(),
            window_seconds: None,
        }
    }

    pub fn with_burst(limit: u64) -> Self {
        Self {
            limit,
            use_burst: true,
            behavior: RateLimitBehavior::default(),
            window_seconds: None,
        }
    }

    pub fn with_behavior(mut self, behavior: RateLimitBehavior) -> Self {
        self.behavior = behavior;
        self
    }

    pub fn reject_when_disabled(mut self) -> Self {
        self.behavior = RateLimitBehavior::RejectWhenDisabled;
        self
    }

    pub fn with_window(mut self, window_seconds: u64) -> Self {
        self.window_seconds = Some(window_seconds);
        self
    }
}

/// Handle behavior when rate limiting is disabled or unavailable
async fn handle_disabled_behavior(
    behavior: RateLimitBehavior,
    next: Next,
    request: Request,
) -> Result<Response, Response> {
    match behavior {
        RateLimitBehavior::AllowWhenDisabled => Ok(next.run(request).await),
        RateLimitBehavior::RejectWhenDisabled => {
            let body = Json(StatusResponse {
                success: false,
                code: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                message: "Rate limiting service is currently unavailable.".to_string(),
            });
            Err((StatusCode::SERVICE_UNAVAILABLE, body).into_response())
        }
    }
}

/// Extract client IP from request
fn extract_client_ip(
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> String {
    // Try X-Forwarded-For header first (for proxies/load balancers)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fall back to connection info
    if let Some(ConnectInfo(addr)) = connect_info {
        return addr.ip().to_string();
    }

    // Last resort
    "unknown".to_string()
}

/// Create rate limit response headers
fn create_rate_limit_headers(info: &RateLimit) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        "X-RateLimit-Limit",
        HeaderValue::from_str(&info.limit.to_string()).unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        HeaderValue::from_str(&info.remaining.to_string()).unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset",
        HeaderValue::from_str(&info.reset_at.to_string()).unwrap(),
    );

    if let Some(retry_after) = info.retry_after {
        headers.insert(
            "Retry-After",
            HeaderValue::from_str(&retry_after.to_string()).unwrap(),
        );
    }

    headers
}

/// Configuration for rate limiting check
enum RateLimitConfig {
    Global(RateLimitBehavior),
    Endpoint(EndpointRateLimit),
}

/// Generic rate limiting logic
async fn apply_rate_limit(
    state: Arc<RateLimitState>,
    config: RateLimitConfig,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let (behavior, limit, use_burst, window_seconds, key_prefix, log_msg) = match &config {
        RateLimitConfig::Global(behavior) => (
            *behavior,
            None,
            false,
            state.window_seconds,
            "global",
            "global rate limit",
        ),
        RateLimitConfig::Endpoint(cfg) => (
            cfg.behavior,
            Some(cfg.limit),
            cfg.use_burst,
            cfg.window_seconds.unwrap_or(state.window_seconds),
            "endpoint",
            "endpoint rate limit",
        ),
    };

    // If rate limiting is disabled, check behavior
    if !state.enabled {
        return handle_disabled_behavior(behavior, next, request).await;
    }

    let limiter = match &state.limiter {
        Some(l) => l,
        None => return handle_disabled_behavior(behavior, next, request).await,
    };

    let client_ip = extract_client_ip(request.headers(), connect_info.as_ref());
    let path = request.uri().path().to_string();
    let key = if key_prefix == "global" {
        format!("global:{}", client_ip)
    } else {
        format!("endpoint:{}:{}", path, client_ip)
    };

    debug!(
        ip = %client_ip,
        path = %path,
        limit = ?limit,
        window_seconds = window_seconds,
        "Checking {}",
        log_msg
    );

    // Check rate limit
    let rate_limit_info = if use_burst {
        match limiter
            .check_rate_limit_with_burst(&key, limit, Some(window_seconds))
            .await
        {
            Ok(info) => info,
            Err(e) => {
                warn!("Rate limit check failed: {}", e);
                return Ok(next.run(request).await);
            }
        }
    } else {
        match limiter
            .check_rate_limit(&key, limit, Some(window_seconds))
            .await
        {
            Ok(info) => info,
            Err(e) => {
                warn!("Rate limit check failed: {}", e);
                return Ok(next.run(request).await);
            }
        }
    };

    if !rate_limit_info.allowed {
        warn!(
            ip = %client_ip,
            path = %path,
            limit = ?limit,
            "{} exceeded",
            log_msg
        );

        let headers = create_rate_limit_headers(&rate_limit_info);

        // Generate dynamic time unit description using the actual window_seconds
        let time_description = match window_seconds {
            1 => "per second".to_string(),
            60 => "per minute".to_string(),
            3600 => "per hour".to_string(),
            86400 => "per day".to_string(),
            seconds => format!("per {} seconds", seconds),
        };

        let message = if let Some(limit) = limit {
            format!(
                "Too many requests to this endpoint. Limit: {} requests {}.",
                limit, time_description
            )
        } else {
            "Too many requests. Please try again later.".to_string()
        };

        let body = Json(json!({
            "success": false,
            "code": 429,
            "message": message,
            "retry_after": rate_limit_info.retry_after,
        }));

        return Err((StatusCode::TOO_MANY_REQUESTS, headers, body).into_response());
    }

    // Add rate limit headers to response
    let mut response = next.run(request).await;
    let response_headers = response.headers_mut();
    let rate_headers = create_rate_limit_headers(&rate_limit_info);

    for (key, value) in rate_headers {
        if let Some(key) = key {
            response_headers.insert(key, value);
        }
    }

    Ok(response)
}

/// Global rate limiting middleware (applies to all requests)
pub async fn rate_limit_middleware(
    State(state): State<Arc<RateLimitState>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let behavior = state.global_behavior;
    apply_rate_limit(
        state,
        RateLimitConfig::Global(behavior),
        connect_info,
        request,
        next,
    )
    .await
}

/// Endpoint-specific rate limiting middleware
pub async fn local_rate_limit_middleware(
    State(state): State<Arc<RateLimitState>>,
    config: Option<Extension<EndpointRateLimit>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // If no config provided, use state's endpoint behavior
    let config = config
        .map(|ext| {
            debug!("Rate limit config found: limit={}, window={:?}", ext.0.limit, ext.0.window_seconds);
            ext.0
        })
        .unwrap_or_else(|| {
            warn!("No rate limit config found in Extension, using fallback");
            EndpointRateLimit {
                limit: 0, // Will use default from state
                use_burst: false,
                behavior: state.endpoint_behavior,
                window_seconds: None,
            }
        });

    apply_rate_limit(
        state,
        RateLimitConfig::Endpoint(config),
        connect_info,
        request,
        next,
    )
    .await
}
