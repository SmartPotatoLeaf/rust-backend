use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::models::user::{FullUserResponse, UserResponse};
use crate::adapters::web::state::AppState;
use axum::{response::IntoResponse, routing::get, Json, Router};
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(me),
    components(schemas(UserResponse, FullUserResponse)),
    tags((name = "users", description = "User endpoints")),
    security(("jwt_auth" = []))
)]
pub struct UserApi;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/users/me", get(me))
}

#[utoipa::path(
    get,
    path = "/users/me",
    responses(
        (status = 200, description = "Get current user", body = FullUserResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "users"
)]
async fn me(AuthUser(user): AuthUser) -> impl IntoResponse {
    Json(FullUserResponse::from(user))
}
