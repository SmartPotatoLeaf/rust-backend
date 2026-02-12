use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::models::{
    auth::{LoginRequest, RegisterRequest, TokenResponse},
    health::HealthResponse,
    user::UserResponse,
};
use spl_shared::http::responses::StatusResponse;

use crate::adapters::web::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use spl_shared::http::extractor::ValidatedJson;
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(login, register, health_check, validate),
    components(schemas(LoginRequest, TokenResponse, RegisterRequest, UserResponse, HealthResponse, StatusResponse)),
    tags((name = "auth", description = "Authentication endpoints"))
)]
pub struct AuthApi;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/health", get(health_check))
        .route("/auth/validate", post(validate))
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
    match state
        .auth_service
        .login(&payload.username, &payload.password, payload.company_id)
        .await
    {
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
