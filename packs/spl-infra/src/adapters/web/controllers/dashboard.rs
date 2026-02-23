use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{json_if, StatusResponse};

use crate::adapters::web::{
    middleware::auth::AuthUser,
    models::{
        common::SimplifiedQuery,
        dashboard::{
            DashboardCountsRequest, DashboardCountsResponse, DashboardDistributionResponse,
            DashboardFiltersRequest, DashboardFiltersResponse, DashboardLabelCountResponse,
            DashboardSummaryRequest, DashboardSummaryResponse, SimplifiedDashboardFiltersResponse,
        },
        user::SimplifiedUserResponse,
    },
    state::AppState,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};

use crate::adapters::web::middleware::permissions::{
    permission_check, RequiredRoles, RoleValidation,
};
use crate::adapters::web::models::dashboard::SimplifiedDashboardCountsResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum DashboardFiltersOrSimplifiedResponse {
    Full(DashboardFiltersResponse),
    Simplified(SimplifiedDashboardFiltersResponse),
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum DashboardCountsOrSimplifiedResponse {
    Full(DashboardCountsResponse),
    Simplified(SimplifiedDashboardCountsResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(get_filters, get_summary, get_filters_admin, get_summary_counts),
    components(schemas(
        DashboardFiltersRequest,
        DashboardFiltersResponse,
        SimplifiedDashboardFiltersResponse,
        DashboardFiltersOrSimplifiedResponse,
        DashboardSummaryRequest,
        DashboardSummaryResponse,
        StatusResponse,
        SimplifiedUserResponse,
        DashboardDistributionResponse,
        DashboardLabelCountResponse,
        DashboardCountsRequest,
        DashboardCountsResponse,
        SimplifiedDashboardCountsResponse,
        DashboardCountsOrSimplifiedResponse
    )),
    tags((name = "dashboard", description = "Dashboard analytics endpoints"))
)]
pub struct DashboardApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    Router::new()
        .route("/dashboard/filters", get(get_filters))
        .route(
            "/dashboard/filters",
            post(get_filters_admin)
                .route_layer(admin_only_layer)
                .route_layer(admin_extension_roles),
        )
        .route("/dashboard/summary", post(get_summary))
        .route("/dashboard/counts", post(get_summary_counts))
        .with_state(state)
}

async fn _get_filters(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(simplified_query): Query<SimplifiedQuery>,
    payload: DashboardFiltersRequest,
) -> Result<impl IntoResponse> {
    let filters = state
        .dashboard_service
        .get_filters(user, payload.into())
        .await?;

    let response = json_if(
        filters,
        simplified_query.simplified,
        SimplifiedDashboardFiltersResponse::from,
        DashboardFiltersResponse::from,
    );

    Ok((StatusCode::OK, response))
}

#[utoipa::path(
    get,
    path = "/dashboard/filters",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Available filters for dashboard", body = DashboardFiltersOrSimplifiedResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "dashboard"
)]
async fn get_filters(
    state: State<Arc<AppState>>,
    user: AuthUser,
    simplified_query: Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    _get_filters(
        state,
        user,
        simplified_query,
        DashboardFiltersRequest { company_id: None },
    )
    .await
}

#[utoipa::path(
    post,
    path = "/dashboard/filters",
    request_body = DashboardFiltersRequest,
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Available filters for dashboard", body = DashboardFiltersOrSimplifiedResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "dashboard"
)]
async fn get_filters_admin(
    state: State<Arc<AppState>>,
    user: AuthUser,
    simplified_query: Query<SimplifiedQuery>,
    ValidatedJson(payload): ValidatedJson<DashboardFiltersRequest>,
) -> Result<impl IntoResponse> {
    _get_filters(state, user, simplified_query, payload).await
}

#[utoipa::path(
    post,
    path = "/dashboard/summary",
    request_body = DashboardSummaryRequest,
    responses(
        (status = 200, description = "Dashboard summary with statistics", body = DashboardSummaryResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "dashboard"
)]
async fn get_summary(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<DashboardSummaryRequest>,
) -> Result<impl IntoResponse> {
    let summary = state
        .dashboard_service
        .get_summary(user, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(DashboardSummaryResponse::from(summary)),
    ))
}

#[utoipa::path(
    post,
    path = "/dashboard/counts",
    request_body = DashboardCountsRequest,
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Dashboard summary with statistics and last predictions", body = DashboardCountsOrSimplifiedResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "dashboard"
)]
async fn get_summary_counts(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(simplified_query): Query<SimplifiedQuery>,
    ValidatedJson(payload): ValidatedJson<DashboardCountsRequest>,
) -> Result<impl IntoResponse> {
    let counts = state
        .dashboard_service
        .get_counts(user, payload.into())
        .await?;

    let results = json_if(
        counts,
        simplified_query.simplified,
        SimplifiedDashboardCountsResponse::from,
        DashboardCountsResponse::from,
    );

    Ok((StatusCode::OK, results))
}
