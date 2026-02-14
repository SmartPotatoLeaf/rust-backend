use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::middleware::permissions::{
    permission_check, RequiredRoles, RoleValidation,
};
use crate::adapters::web::models::feedback::status::{CreateFeedbackStatusRequest, FeedbackStatusResponse, SimplifiedFeedbackStatusResponse, UpdateFeedbackStatusRequest};
use crate::adapters::web::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{ok_if_or_not_found, ok_iter_if_or_not_found, StatusResponse};
use std::sync::Arc;
use axum::extract::Query;
use utoipa::OpenApi;
use crate::adapters::web::models::common::SimplifiedQuery;

#[derive(OpenApi)]
#[openapi(
    paths(get_all, get_by_id, create, update, delete_status),
    components(schemas(
        CreateFeedbackStatusRequest,
        UpdateFeedbackStatusRequest,
        FeedbackStatusResponse,
        SimplifiedFeedbackStatusResponse,
        StatusResponse
    )),
    tags((name = "feedback_status", description = "Feedback Status endpoints")),
    security(("jwt_auth" = []))
)]
pub struct FeedbackStatusApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    Router::new()
        .route("/feedback/statuses", get(get_all).post(create))
        .route(
            "/feedback/statuses/{id}",
            get(get_by_id).put(update).delete(delete_status),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/feedback/statuses",
    params(
        ("simplified" = Option<bool>, Query, description = "Return simplified response")
    ),
    responses(
        (status = 200, description = "List all feedback statuses", body = Vec<FeedbackStatusResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedback_status"
)]
async fn get_all(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let statuses = state.feedback_status_service.get_all().await?;

    ok_iter_if_or_not_found(
        statuses,
        query.simplified,
        SimplifiedFeedbackStatusResponse::from,
        FeedbackStatusResponse::from,
        || "There are no feedback statuses available".to_string(),
    )
}

#[utoipa::path(
    get,
    path = "/feedback/statuses/{id}",
    params(
        ("id" = i32, Path, description = "Feedback Status ID"),
        ("simplified" = Option<bool>, Query, description = "Return simplified response")
    ),
    responses(
        (status = 200, description = "Feedback Status found", body = FeedbackStatusResponse),
        (status = 404, description = "Feedback Status not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedback_status"
)]
async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    _user: AuthUser,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state.feedback_status_service.get_by_id(id).await?;

    ok_if_or_not_found(
        result,
        query.simplified,
        SimplifiedFeedbackStatusResponse::from,
        FeedbackStatusResponse::from,
        move || format!("The feedback status with id {} does not exist", id),
    )
}

#[utoipa::path(
    post,
    path = "/feedback/statuses",
    request_body = CreateFeedbackStatusRequest,
    responses(
        (status = 201, description = "Feedback Status created successfully", body = FeedbackStatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedback_status"
)]
async fn create(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateFeedbackStatusRequest>,
) -> Result<impl IntoResponse> {
    let created = state.feedback_status_service.create(payload.into()).await?;
    Ok((StatusCode::CREATED, Json(FeedbackStatusResponse::from(created))))
}

#[utoipa::path(
    put,
    path = "/feedback/statuses/{id}",
    params(
        ("id" = i32, Path, description = "Feedback Status ID")
    ),
    request_body = UpdateFeedbackStatusRequest,
    responses(
        (status = 200, description = "Feedback Status updated successfully", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 404, description = "Feedback Status not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedback_status"
)]
async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateFeedbackStatusRequest>,
) -> Result<impl IntoResponse> {
    let _ = state.feedback_status_service.update(id, payload.into()).await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Feedback Status updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/feedback/statuses/{id}",
    params(
        ("id" = i32, Path, description = "Feedback Status ID")
    ),
    responses(
        (status = 200, description = "Feedback Status deleted successfully", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 404, description = "Feedback Status not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedback_status"
)]
async fn delete_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let _ = state.feedback_status_service.delete(id).await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Feedback Status deleted successfully".to_string(),
        }),
    ))
}
