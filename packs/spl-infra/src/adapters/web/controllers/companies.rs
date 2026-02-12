use spl_shared::error::{AppError, Result};
use spl_shared::http::extractor::ValidatedJson;

use crate::adapters::web::{
    middleware::{
        auth::AuthUser,
        permissions::{permission_check, RequiredRoles, RoleValidation},
    },
    models::{
        common::SimplifiedQuery,
        company::{
            CompanyResponse, CreateCompanyRequest, SimplifiedCompanyResponse, UpdateCompanyRequest,
        },
    },
    state::AppState,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
    Extension, Json, Router,
};

use serde::{Deserialize, Serialize};
use spl_shared::http::responses::{conditional_json, StatusResponse};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum CompanyOrSimplifiedResponse {
    Company(CompanyResponse),
    Simplified(SimplifiedCompanyResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(create_company, get_company, update_company, delete_company),
    components(schemas(
        CreateCompanyRequest, 
        CompanyResponse, 
        UpdateCompanyRequest, 
        SimplifiedCompanyResponse, 
        CompanyOrSimplifiedResponse,
        StatusResponse
    )),
    tags((name = "companies", description = "Company endpoints"))
)]
pub struct CompaniesApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    let admin_router = Router::new()
        .route("/companies", post(create_company))
        .route(
            "/companies/:id",
            put(update_company).delete(delete_company),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state.clone());

    let public_router = Router::new()
        .route("/companies/:id", get(get_company))
        .with_state(state);

    Router::new().merge(public_router).merge(admin_router)
}

#[utoipa::path(
    post,
    path = "/companies",
    request_body = CreateCompanyRequest,
    responses(
        (status = 201, description = "Company created successfully", body = CompanyResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 409, description = "Company already exists", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "companies"
)]
async fn create_company(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateCompanyRequest>,
) -> Result<impl IntoResponse> {
    let result = state.company_service.create(&user, payload.into()).await?;

    Ok((StatusCode::CREATED, Json(CompanyResponse::from(result))))
}

#[utoipa::path(
    get,
    path = "/companies/{id}",
    params(
        ("id" = Uuid, Path, description = "Company ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Company details", body = CompanyOrSimplifiedResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Access denied", body = StatusResponse),
        (status = 404, description = "Company not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "companies"
)]
async fn get_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<SimplifiedQuery>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let result = state.company_service.get_by_id(&user, id).await?;

    if result.is_none() {
        Err(AppError::NotFound(format!("Company {} not found", id)))
    } else {
        Ok((
            StatusCode::OK,
            conditional_json::<_, SimplifiedCompanyResponse, CompanyResponse>(
                result.unwrap(),
                query.simplified,
            ),
        ))
    }
}

#[utoipa::path(
    put,
    path = "/companies/{id}",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    request_body = UpdateCompanyRequest,
    responses(
        (status = 200, description = "Company updated", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Access denied", body = StatusResponse),
        (status = 404, description = "Company not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "companies"
)]
async fn update_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateCompanyRequest>,
) -> Result<impl IntoResponse> {
    // Validation

    let _ = state
        .company_service
        .update(&user, id, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Company updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/companies/{id}",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Company deleted", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden - Admin access required", body = StatusResponse),
        (status = 404, description = "Company not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "companies"
)]
async fn delete_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let _ = state.company_service.delete(&user, id).await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Company deleted successfully".to_string(),
        }),
    ))
}
