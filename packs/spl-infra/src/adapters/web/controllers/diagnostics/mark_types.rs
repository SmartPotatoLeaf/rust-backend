use spl_shared::error::{AppError, Result};
use spl_shared::http::extractor::ValidatedJson;

use crate::adapters::web::{
    middleware::{
        auth::AuthUser,
        permissions::{permission_check, RequiredRoles, RoleValidation},
    },
    models::{
        common::SimplifiedQuery,
        diagnostics::{
            CreateMarkTypeRequest, MarkTypeResponse, SimplifiedMarkTypeResponse,
            UpdateMarkTypeRequest,
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

use spl_shared::http::responses::{conditional_iter_json, conditional_json, StatusResponse};
use std::sync::Arc;
use axum::routing::{post, put};
use utoipa::{OpenApi, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum MarkTypeOrSimplifiedResponse {
    MarkType(MarkTypeResponse),
    Simplified(SimplifiedMarkTypeResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(get_all_mark_types, get_mark_type, create_mark_type, update_mark_type, delete_mark_type),
    components(schemas(
        CreateMarkTypeRequest, 
        MarkTypeResponse, 
        UpdateMarkTypeRequest, 
        SimplifiedMarkTypeResponse, 
        MarkTypeOrSimplifiedResponse,
        StatusResponse
    )),
    tags((name = "diagnostics/marks/types", description = "Diagnostic mark type endpoints"))
)]
pub struct MarkTypesApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    let admin_router = Router::new()
        .route("/diagnostics/marks/types", post(create_mark_type))
        .route(
            "/diagnostics/marks/types/:id",
            put(update_mark_type).delete(delete_mark_type),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state.clone());

    let public_router = Router::new()
        .route(
            "/diagnostics/marks/types",
            get(get_all_mark_types),
        )
        .route(
            "/diagnostics/marks/types/:id",
            get(get_mark_type),
        )
        .with_state(state);

    Router::new().merge(public_router).merge(admin_router)
}

#[utoipa::path(
    get,
    path = "/diagnostics/marks/types",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "List all mark types", body = Vec<MarkTypeOrSimplifiedResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 404, description = "No mark types found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/marks/types"
)]
async fn get_all_mark_types(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SimplifiedQuery>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let mark_types = state.mark_type_service.get_all(&user).await?;

    if mark_types.is_empty() {
        Err(AppError::NoContent(
            "There are no mark types available".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_iter_json::<_, SimplifiedMarkTypeResponse, MarkTypeResponse>(
                mark_types,
                query.simplified,
            ),
        ))
    }
}

#[utoipa::path(
    get,
    path = "/diagnostics/marks/types/{id}",
    params(
        ("id" = i32, Path, description = "Mark Type ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Mark type found", body = MarkTypeOrSimplifiedResponse),
        (status = 404, description = "Mark type not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/marks/types"
)]
async fn get_mark_type(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<i32>,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state.mark_type_service.get_by_id(&user, id).await?;

    if result.is_none() {
        Err(AppError::NotFound(
            "The mark type with the specified ID does not exist".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_json::<_, SimplifiedMarkTypeResponse, MarkTypeResponse>(
                result.unwrap(),
                query.simplified,
            ),
        ))
    }
}

#[utoipa::path(
    post,
    path = "/diagnostics/marks/types",
    request_body = CreateMarkTypeRequest,
    responses(
        (status = 201, description = "Mark type created successfully", body = MarkTypeResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/marks/types"
)]
async fn create_mark_type(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateMarkTypeRequest>,
) -> Result<impl IntoResponse> {
    let result = state
        .mark_type_service
        .create(&user, payload.into())
        .await?;

    Ok((StatusCode::CREATED, Json(MarkTypeResponse::from(result))))
}

#[utoipa::path(
    put,
    path = "/diagnostics/marks/types/{id}",
    params(("id" = i32, Path, description = "Mark Type ID")),
    request_body = UpdateMarkTypeRequest,
    responses(
        (status = 200, description = "Mark type updated successfully", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 404, description = "Mark type not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/marks/types"
)]
async fn update_mark_type(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateMarkTypeRequest>,
) -> Result<impl IntoResponse> {
    let _ = state
        .mark_type_service
        .update(&user, id, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Mark type updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/diagnostics/marks/types/{id}",
    params(("id" = i32, Path, description = "Mark Type ID")),
    responses(
        (status = 200, description = "Mark type deleted successfully", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 404, description = "Mark type not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/marks/types"
)]
async fn delete_mark_type(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let _ = state.mark_type_service.delete(&user, id).await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Mark type deleted successfully".to_string(),
        }),
    ))
}
