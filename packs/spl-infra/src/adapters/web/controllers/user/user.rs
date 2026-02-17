use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::middleware::permissions::{
    permission_check, RequiredRoles, RoleValidation,
};
use crate::adapters::web::models::user::{
    ChangePasswordRequest, FullUserResponse, UpdateProfileRequest, UpdateUserRequest, UserResponse,
};
use crate::adapters::web::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, put},
    Extension, Json, Router,
};
use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::StatusResponse;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(OpenApi)]
#[openapi(
    paths(me, update_profile, change_password, update_user, delete_user),
    components(schemas(
        UserResponse,
        FullUserResponse,
        UpdateUserRequest,
        UpdateProfileRequest,
        ChangePasswordRequest,
        StatusResponse
    )),
    tags((name = "users", description = "User endpoints")),
    security(("jwt_auth" = []))
)]
pub struct UserApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    Router::new()
        .route("/users/me", get(me).put(update_profile))
        .route("/users/me/password", put(change_password))
        .route(
            "/users/{id}",
            put(update_user)
                .delete(delete_user)
                .route_layer(admin_only_layer)
                .route_layer(admin_extension_roles),
        )
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

#[utoipa::path(
    put,
    path = "/users/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = FullUserResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "users"
)]
async fn update_profile(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateProfileRequest>,
) -> Result<impl IntoResponse> {
    let updated = state
        .user_service
        .update_profile(&user, payload.into())
        .await?;

    Ok((StatusCode::OK, Json(FullUserResponse::from(updated))))
}

#[utoipa::path(
    put,
    path = "/users/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized / Invalid current password", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "users"
)]
async fn change_password(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<ChangePasswordRequest>,
) -> Result<impl IntoResponse> {
    state
        .user_service
        .change_password(&user, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Password changed successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Insufficient permissions", body = StatusResponse),
        (status = 404, description = "User not found", body = StatusResponse),
        (status = 409, description = "Username already exists", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "users"
)]
async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(requester): AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse> {
    state
        .user_service
        .update_user(&requester, id, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "User updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Insufficient permissions", body = StatusResponse),
        (status = 404, description = "User not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "users"
)]
async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(requester): AuthUser,
) -> Result<impl IntoResponse> {
    state.user_service.delete_user(&requester, id).await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "User deleted successfully".to_string(),
        }),
    ))
}
