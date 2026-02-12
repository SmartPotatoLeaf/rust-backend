use crate::adapters::web::mappers::plot::CreatePlotContext;
use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::middleware::permissions::{
    permission_check, RequiredRoles, RoleValidation,
};
use crate::adapters::web::models::{
    common::SimplifiedQuery,
    plot::{
        AssignPredictionsRequest, AssignedPlotResponse, CreatePlotRequest, DetailedPlotResponse,
        DetailedPlotsRequest, DetailedPlotsResponse, PlotResponse, SimplifiedPlotResponse,
        UpdatePlotRequest,
    },
};
use crate::adapters::web::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{middleware, response::IntoResponse, Extension, Json, Router};
use serde::{Deserialize, Serialize};
use spl_shared::error::{AppError, Result};
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{
    conditional_iter_json, conditional_json, ok_or_not_found, StatusResponse,
};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum PlotOrSimplifiedResponse {
    Plot(PlotResponse),
    Simplified(SimplifiedPlotResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_plots,
        create_plot,
        get_plot,
        update_plot,
        delete_plot,
        assign_predictions,
        unassign_predictions,
        get_detailed_plots,
        get_detailed_plot,
        get_default_detailed
    ),
    components(schemas(
        CreatePlotRequest,
        UpdatePlotRequest,
        AssignPredictionsRequest,
        DetailedPlotsRequest,
        PlotResponse,
        SimplifiedPlotResponse,
        PlotOrSimplifiedResponse,
        DetailedPlotResponse,
        DetailedPlotsResponse,
        AssignedPlotResponse,
        StatusResponse
    )),
    tags((name = "Plots", description = "Plot management endpoints")),
    security(
        ("jwt_auth" = [])
    ),
)]
pub struct PlotsApi;

/// Create plots router with all endpoints
pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let permission_layer = middleware::from_fn_with_state(state.clone(), permission_check);

    let supervisor_lower_extension_roles = Extension(RequiredRoles(
        vec!["supervisor".to_string()],
        RoleValidation::Lower,
    ));

    let supervisor_only_extension_roles = Extension(RequiredRoles(
        vec!["supervisor".to_string()],
        RoleValidation::SameStrict,
    ));

    let supervisor_lower_router = Router::new()
        .route("/plots", get(get_plots))
        .route("/plots/:id", get(get_plot))
        .route("/plots/:id/assign", post(assign_predictions))
        .route("/plots/unassign", post(unassign_predictions))
        .route("/plots/detailed", post(get_detailed_plots))
        .route("/plots/detailed/:id", get(get_detailed_plot))
        .route("/plots/default/detailed", get(get_default_detailed))
        .route_layer(permission_layer.clone())
        .route_layer(supervisor_lower_extension_roles.clone())
        .with_state(state.clone());

    let supervisor_only_router = Router::new()
        .route("/plots", post(create_plot))
        .route("/plots/:id", put(update_plot).delete(delete_plot))
        .route_layer(permission_layer.clone())
        .route_layer(supervisor_only_extension_roles.clone())
        .with_state(state);

    Router::new()
        .merge(supervisor_lower_router)
        .merge(supervisor_only_router)
}

/// Get all plots for authenticated user
#[utoipa::path(
    get,
    path = "/plots",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "List of plots", body = Vec<PlotOrSimplifiedResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No plots found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn get_plots(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SimplifiedQuery>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let plots = state.plot_service.get_all_by_user(&user, None).await?;

    if plots.is_empty() {
        Err(AppError::NoContent(
            "There are no plots available".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_iter_json::<_, SimplifiedPlotResponse, PlotResponse>(
                plots,
                query.simplified,
            ),
        ))
    }
}

/// Get a single plot by ID
#[utoipa::path(
    get,
    path = "/plots/{id}",
    params(
        ("id" = Uuid, Path, description = "Plot ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Plot details", body = PlotOrSimplifiedResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Plot not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn get_plot(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state.plot_service.get_by_id(&user, id, None).await?;

    if result.is_none() {
        Err(AppError::NotFound(
            "The plot with the specified ID does not exist".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_json::<_, SimplifiedPlotResponse, PlotResponse>(
                result.unwrap(),
                query.simplified,
            ),
        ))
    }
}

/// Create a new plot
#[utoipa::path(
    post,
    path = "/plots",
    request_body = CreatePlotRequest,
    responses(
        (status = 201, description = "Plot created", body = PlotResponse),
        (status = 400, description = "Validation error", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn create_plot(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(req): ValidatedJson<CreatePlotRequest>,
) -> Result<impl IntoResponse> {
    let dto = req.into_with_context(CreatePlotContext {
        company_id: user.company.as_ref().map(|c| c.id),
    })?;
    let plot = state.plot_service.create(&user, dto).await?;
    Ok((StatusCode::CREATED, Json(PlotResponse::from(plot))))
}

/// Update an existing plot
#[utoipa::path(
    put,
    path = "/plots/{id}",
    params(
        ("id" = Uuid, Path, description = "Plot ID")
    ),
    request_body = UpdatePlotRequest,
    responses(
        (status = 200, description = "Plot updated", body = StatusResponse),
        (status = 400, description = "Validation error", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Plot not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn update_plot(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdatePlotRequest>,
) -> Result<impl IntoResponse> {
    let _ = state
        .plot_service
        .update(&user, id, req.into(), None)
        .await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Plot updated successfully".to_string(),
        }),
    ))
}

/// Delete a plot
#[utoipa::path(
    delete,
    path = "/plots/{id}",
    params(
        ("id" = Uuid, Path, description = "Plot ID")
    ),
    responses(
        (status = 200, description = "Plot deleted", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Plot not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn delete_plot(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let _ = state.plot_service.delete(&user, id, None).await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Plot deleted successfully".to_string(),
        }),
    ))
}

/// Assign predictions to a plot
#[utoipa::path(
    post,
    path = "/plots/{id}/assign",
    params(
        ("id" = Uuid, Path, description = "Plot ID")
    ),
    request_body = AssignPredictionsRequest,
    responses(
        (status = 200, description = "Predictions assigned", body = AssignPredictionsResponse),
        (status = 400, description = "Validation error", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Plot or predictions not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn assign_predictions(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<AssignPredictionsRequest>,
) -> Result<impl IntoResponse> {
    let result = state
        .plot_service
        .assign_predictions(&user, id, req.into(), None)
        .await?;
    Ok((
        StatusCode::OK,
        Json(AssignedPlotResponse {
            prediction_ids: result.prediction_ids,
        }),
    ))
}

/// Unassign predictions from plots
#[utoipa::path(
    post,
    path = "/plots/unassign",
    request_body = AssignPredictionsRequest,
    responses(
        (status = 200, description = "Predictions unassigned", body = AssignPredictionsResponse),
        (status = 400, description = "Validation error", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Predictions not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn unassign_predictions(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(req): ValidatedJson<AssignPredictionsRequest>,
) -> Result<impl IntoResponse> {
    let result = state
        .plot_service
        .unassign_predictions(&user, req.into())
        .await?;
    Ok((
        StatusCode::OK,
        Json(AssignedPlotResponse {
            prediction_ids: result.prediction_ids,
        }),
    ))
}

/// Get paginated detailed plots with statistics
#[utoipa::path(
    post,
    path = "/plots/detailed",
    request_body = DetailedPlotsRequest,
    responses(
        (status = 200, description = "Paginated detailed plots", body = PaginatedDetailedPlotResponse),
        (status = 400, description = "Validation error", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn get_detailed_plots(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(req): ValidatedJson<DetailedPlotsRequest>,
) -> Result<impl IntoResponse> {
    let result = state
        .plot_service
        .get_detailed(&user, req.into(), None)
        .await?;
    Ok((StatusCode::OK, Json(DetailedPlotsResponse::from(result))))
}

/// Get detailed statistics for a specific plot
#[utoipa::path(
    get,
    path = "/plots/detailed/{id}",
    params(
        ("id" = Uuid, Path, description = "Plot ID")
    ),
    responses(
        (status = 200, description = "Detailed plot statistics", body = DetailedPlotWebResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Plot not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn get_detailed_plot(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let result = state
        .plot_service
        .get_detailed_by_id(&user, id, None, vec![])
        .await?;

    ok_or_not_found(
        result.map(DetailedPlotResponse::from),
        "The plot with the specified ID does not exist".to_string(),
    )
}

/// Get statistics for unassigned predictions (default plot)
#[utoipa::path(
    get,
    path = "/plots/default/detailed",
    responses(
        (status = 200, description = "Default plot statistics", body = DetailedPlotWebResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No unassigned predictions", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Plots"
)]
async fn get_default_detailed(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let result = state
        .plot_service
        .get_default_detailed(&user, None, vec![])
        .await?;

    ok_or_not_found(
        result.map(DetailedPlotResponse::from),
        "No unassigned predictions found for the default plot".to_string(),
    )
}
