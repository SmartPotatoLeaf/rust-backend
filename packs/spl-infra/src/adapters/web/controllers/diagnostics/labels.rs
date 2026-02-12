use spl_shared::error::{AppError, Result};
use spl_shared::http::extractor::ValidatedJson;

use crate::adapters::web::{
    middleware::auth::AuthUser,
    models::{
        common::SimplifiedQuery,
        diagnostics::{
            CreateLabelRequest, LabelResponse, SimplifiedLabelResponse, UpdateLabelRequest,
        },
    },
    state::AppState,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};

use crate::adapters::web::middleware::permissions::{
    permission_check, RequiredRoles, RoleValidation,
};
use axum::routing::{post, put};
use serde::{Deserialize, Serialize};
use spl_shared::http::responses::{conditional_iter_json, conditional_json, StatusResponse};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(get_all_labels, get_label, create_label, update_label, delete_label),
    components(schemas(
        CreateLabelRequest,
        LabelResponse,
        UpdateLabelRequest,
        SimplifiedLabelResponse,
        LabelOrSimplifiedResponse,
        StatusResponse
    )),
    tags((name = "diagnostics/labels", description = "Diagnostic label endpoints"))
)]
pub struct LabelsApi;

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum LabelOrSimplifiedResponse {
    Label(LabelResponse),
    Simplified(SimplifiedLabelResponse),
}

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    let admin_router = Router::new()
        .route("/diagnostics/labels", post(create_label))
        .route(
            "/diagnostics/labels/:id",
            put(update_label).delete(delete_label),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state.clone());

    let public_router = Router::new()
        .route("/diagnostics/labels", get(get_all_labels))
        .route("/diagnostics/labels/:id", get(get_label))
        .with_state(state);

    Router::new().merge(public_router).merge(admin_router)
}

#[utoipa::path(
    get,
    path = "/diagnostics/labels",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "List all labels (can be simplified)", body = Vec<LabelOrSimplifiedResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No labels found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/labels"
)]
async fn get_all_labels(
    State(state): State<Arc<AppState>>,
    _auth: AuthUser,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let labels = state.label_service.get_all().await?;

    if labels.is_empty() {
        Err(AppError::NoContent("No labels found".to_string()))
    } else {
        Ok((
            StatusCode::OK,
            conditional_iter_json::<_, SimplifiedLabelResponse, LabelResponse>(
                labels,
                query.simplified,
            ),
        ))
    }
}

#[utoipa::path(
    get,
    path = "/diagnostics/labels/{id}",
    params(
        ("id" = i32, Path, description = "Label ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Label found (can be simplified)", body = LabelOrSimplifiedResponse),
        (status = 404, description = "Label not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/labels"
)]
async fn get_label(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<i32>,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state.label_service.get_by_id(id).await?;

    if result.is_none() {
        Err(AppError::NotFound(format!(
            "Label with id {} not found",
            id
        )))
    } else {
        Ok((
            StatusCode::OK,
            conditional_json::<_, SimplifiedLabelResponse, LabelResponse>(
                result.unwrap(),
                query.simplified,
            ),
        ))
    }
}

#[utoipa::path(
    post,
    path = "/diagnostics/labels",
    request_body = CreateLabelRequest,
    responses(
        (status = 201, description = "Label created successfully", body = LabelResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/labels"
)]
async fn create_label(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateLabelRequest>,
) -> Result<impl IntoResponse> {
    let result = state.label_service.create(&user, payload.into()).await?;

    Ok((StatusCode::CREATED, Json(LabelResponse::from(result))))
}

#[utoipa::path(
    put,
    path = "/diagnostics/labels/{id}",
    params(("id" = i32, Path, description = "Label ID")),
    request_body = UpdateLabelRequest,
    responses(
        (status = 200, description = "Label updated successfully", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 404, description = "Label not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/labels"
)]
async fn update_label(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateLabelRequest>,
) -> Result<impl IntoResponse> {
    let _ = state
        .label_service
        .update(&user, id, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Label updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/diagnostics/labels/{id}",
    params(("id" = i32, Path, description = "Label ID")),
    responses(
        (status = 200, description = "Label deleted successfully", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 404, description = "Label not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/labels"
)]
async fn delete_label(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let _ = state.label_service.delete(&user, id).await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Label deleted successfully".to_string(),
        }),
    ))
}
