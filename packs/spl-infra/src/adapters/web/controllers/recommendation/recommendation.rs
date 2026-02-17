use crate::adapters::web::{
    middleware::{
        auth::AuthUser,
        permissions::{permission_check, RequiredRoles, RoleValidation},
    },
    models::{
        common::SimplifiedQuery,
        recommendation::{
            CreateRecommendationRequest, RecommendationResponse, SimplifiedRecommendationResponse,
            UpdateRecommendationRequest,
        },
    },
};

use crate::adapters::web::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{ok_if_or_not_found, ok_iter_if_or_not_found, StatusResponse};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum RecommendationOrSimplifiedResponse {
    Recommendation(RecommendationResponse),
    Simplified(SimplifiedRecommendationResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(get_all_recommendations, get_recommendation_by_id, get_by_severity, create_recommendation, update_recommendation, delete_recommendation),
    components(schemas(
        CreateRecommendationRequest,
        UpdateRecommendationRequest,
        RecommendationResponse,
        SimplifiedRecommendationResponse,
        RecommendationOrSimplifiedResponse
    )),
    tags((name = "recommendations", description = "Recommendation endpoints")),
    security(("jwt_auth" = []))
)]
pub struct RecommendationApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    let admin_router = Router::new()
        .route("/recommendations", post(create_recommendation))
        .route(
            "/recommendations/{id}",
            put(update_recommendation).delete(delete_recommendation),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state.clone());

    let public_router = Router::new()
        .route("/recommendations", get(get_all_recommendations))
        .route("/recommendations/{id}", get(get_recommendation_by_id))
        .route(
            "/recommendations/severity/{percentage}",
            get(get_by_severity),
        )
        .with_state(state);

    Router::new().merge(public_router).merge(admin_router)
}

#[utoipa::path(
    get,
    path = "/recommendations",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "List all recommendations", body = Vec<RecommendationOrSimplifiedResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No recommendations found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn get_all_recommendations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let recommendations = state.recommendation_service.get_all().await?;

    ok_iter_if_or_not_found(
        recommendations,
        query.simplified,
        SimplifiedRecommendationResponse::from,
        RecommendationResponse::from,
        || "There are no recommendations available".to_string(),
    )
}

#[utoipa::path(
    get,
    path = "/recommendations/{id}",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Recommendation found", body = RecommendationOrSimplifiedResponse),
        (status = 404, description = "Not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn get_recommendation_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let result = state.recommendation_service.get_by_id(id).await?;

    ok_if_or_not_found(
        result,
        query.simplified,
        SimplifiedRecommendationResponse::from,
        RecommendationResponse::from,
        move || format!("The recommendation with id {} does not exist", id),
    )
}

#[utoipa::path(
    get,
    path = "/recommendations/severity/{percentage}",
    params(
        ("percentage" = f64, Path, description = "Severity percentage (0.0 - 100.0)"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Matching recommendations", body = Vec<RecommendationResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No matching recommendations", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn get_by_severity(
    State(state): State<Arc<AppState>>,
    Path(percentage): Path<f32>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let recommendations = state
        .recommendation_service
        .get_by_severity(percentage)
        .await?;

    ok_iter_if_or_not_found(
        recommendations,
        query.simplified,
        SimplifiedRecommendationResponse::from,
        RecommendationResponse::from,
        move || format!("No recommendations found for severity {}", percentage),
    )
}

#[utoipa::path(
    post,
    path = "/recommendations",
    request_body = CreateRecommendationRequest,
    responses(
        (status = 201, description = "Recommendation created", body = RecommendationResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn create_recommendation(
    State(state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateRecommendationRequest>,
) -> Result<impl IntoResponse> {
    let result = state.recommendation_service.create(payload.into()).await?;

    Ok((
        StatusCode::CREATED,
        Json(RecommendationResponse::from(result)),
    ))
}

#[utoipa::path(
    put,
    path = "/recommendations/{id}",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID")
    ),
    request_body = UpdateRecommendationRequest,
    responses(
        (status = 200, description = "Recommendation updated", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 404, description = "Not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn update_recommendation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateRecommendationRequest>,
) -> Result<impl IntoResponse> {
    let _ = state
        .recommendation_service
        .update(id, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Recommendation updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/recommendations/{id}",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID")
    ),
    responses(
        (status = 200, description = "Recommendation deleted", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 404, description = "Not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn delete_recommendation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let _ = state.recommendation_service.delete(id).await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Recommendation deleted successfully".to_string(),
        }),
    ))
}
