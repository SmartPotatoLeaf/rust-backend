use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{json_if, ok_if_or_not_found, StatusResponse};

use crate::adapters::web::{
    middleware::auth::AuthUser,
    models::{
        common::SimplifiedQuery,
        dashboard::{
            DashboardCountsRequest, DashboardCountsResponse, DashboardDetailedPlotResponse,
            DashboardDistributionResponse, DashboardFiltersRequest, DashboardFiltersResponse,
            DashboardLabelCountResponse, DashboardSummaryPlotRequest, DashboardSummaryRequest,
            DashboardSummaryResponse, SimplifiedDashboardFiltersResponse,
        },
        user::SimplifiedUserResponse,
    },
    state::AppState,
};

use axum::{
    extract::{Path, Query, State},
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
use uuid::Uuid;

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
    paths(get_filters, get_summary, get_filters_admin, get_summary_counts, get_summary_detailed_plot_by_id, get_summary_detailed_plot_default),
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
        DashboardCountsOrSimplifiedResponse,
        DashboardSummaryPlotRequest,
        DashboardDetailedPlotResponse
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
                .route_layer(admin_only_layer.clone())
                .route_layer(admin_extension_roles.clone()),
        )
        .route("/dashboard/summary", post(get_summary))
        .route("/dashboard/counts", post(get_summary_counts))
        .route(
            "/dashboard/plots/default/summary",
            post(get_summary_detailed_plot_default),
        )
        .route(
            "/dashboard/plots/{id}/summary",
            post(get_summary_detailed_plot_by_id),
        )
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

#[utoipa::path(
    post,
    path = "/dashboard/plots/default/summary",
    request_body = DashboardSummaryPlotRequest,
    responses(
        (status = 200, description = "Dashboard summary with default plot", body = DashboardDetailedPlotResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 404, description = "No default plot found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "dashboard"
)]
async fn get_summary_detailed_plot_default(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<DashboardSummaryPlotRequest>,
) -> Result<impl IntoResponse> {
    let detailed_plot = state
        .dashboard_service
        .get_summary_detailed_plot_default(user, payload.into())
        .await?;

    ok_if_or_not_found(
        detailed_plot,
        true,
        DashboardDetailedPlotResponse::from,
        DashboardDetailedPlotResponse::from,
        || "No default plot found for company".to_string(),
    )
}

#[utoipa::path(
    post,
    path = "/dashboard/plots/{plot_id}/summary",
    request_body = DashboardSummaryPlotRequest,
    params(
        ("plot_id" = Uuid, Path, description = "Plot UUID")
    ),
    responses(
        (status = 200, description = "Dashboard summary with detailed plot", body = DashboardDetailedPlotResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 404, description = "Plot not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(
        ("jwt_auth" = [])
    ),
    tag = "dashboard"
)]
async fn get_summary_detailed_plot_by_id(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(plot_id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<DashboardSummaryPlotRequest>,
) -> Result<impl IntoResponse> {
    let detailed_plot = state
        .dashboard_service
        .get_summary_detailed_plot_by_id(user, plot_id, payload.into())
        .await?;

    ok_if_or_not_found(
        detailed_plot,
        true,
        DashboardDetailedPlotResponse::from,
        DashboardDetailedPlotResponse::from,
        || "Plot not found".to_string(),
    )
}
