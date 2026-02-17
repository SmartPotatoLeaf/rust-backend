use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::models::{
    auth::{LoginRequest, RegisterRequest, TokenResponse},
    health::HealthResponse,
    user::{SimplifiedRoleResponse, UserResponse},
};
use spl_shared::http::responses::{ok_iter_if_or_not_found, StatusResponse};

use crate::adapters::web::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};

use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::middleware::{
    local_rate_limit_middleware, EndpointRateLimit, RateLimitState,
};
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(login, register, health_check, validate, get_all_roles),
    components(schemas(LoginRequest, TokenResponse, RegisterRequest, UserResponse, HealthResponse, StatusResponse, SimplifiedRoleResponse)),
    tags((name = "auth", description = "Authentication endpoints"))
)]
pub struct AuthApi;

pub fn router(rate_limit_state: Arc<RateLimitState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/auth/login",
            post(login)
                .route_layer(middleware::from_fn_with_state(
                    rate_limit_state,
                    local_rate_limit_middleware,
                ))
                .layer(Extension(EndpointRateLimit::new(5).with_window(60))),
        )
        .route("/auth/register", post(register))
        .route("/auth/health", get(health_check))
        .route("/auth/validate", post(validate))
        .route("/public/roles", get(get_all_roles))
}

#[utoipa::path(
    get,
    path = "/auth/health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    ),
    tag = "auth"
)]
async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Server is clean and running".to_string(),
    })
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = TokenResponse),
        (status = 401, description = "Invalid credentials", body = StatusResponse),
        (status = 404, description = "User not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    tag = "auth"
)]
async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> impl IntoResponse {
    // Validate that at least one identifier is provided
    if let Err(e) = payload.validate_identifier() {
        return (
            StatusCode::BAD_REQUEST,
            Json(StatusResponse {
                success: false,
                code: 400,
                message: e,
            }),
        )
            .into_response();
    }

    match state.auth_service.login(payload.into()).await {
        Ok(token) => (StatusCode::OK, Json(TokenResponse { token })).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 409, description = "Username already exists", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 400, description = "Validation Error", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    tag = "auth"
)]
async fn register(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> impl IntoResponse {
    // Payload validation placeholder

    match state.user_service.create_user(&user, payload.into()).await {
        Ok(user) => (StatusCode::CREATED, Json(UserResponse::from(user))).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/auth/validate",
    responses(
        (status = 200, description = "Token is valid", body = StatusResponse),
        (status = 401, description = "Token is invalid", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "auth"
)]
async fn validate(_user: AuthUser) -> impl IntoResponse {
    Json(StatusResponse {
        success: true,
        code: 200,
        message: "Token is valid".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/public/roles",
    responses(
        (status = 200, description = "List of available roles", body = Vec<SimplifiedRoleResponse>),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    tag = "auth"
)]
async fn get_all_roles(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
    let roles = state.role_service.get_all().await?;

    ok_iter_if_or_not_found(
        roles,
        true,
        SimplifiedRoleResponse::from,
        SimplifiedRoleResponse::from,
        || "There are no roles available".to_string(),
    )
}
