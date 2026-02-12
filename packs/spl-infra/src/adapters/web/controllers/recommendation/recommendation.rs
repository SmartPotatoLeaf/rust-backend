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
use spl_shared::error::{AppError, Result};
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{conditional_iter_json, conditional_json};
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
    paths(get_all, get_by_id, get_by_severity, create, update, delete_recommendation),
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
        .route("/recommendations", post(create))
        .route(
            "/recommendations/:id",
            put(update).delete(delete_recommendation),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state.clone());

    let public_router = Router::new()
        .route("/recommendations", get(get_all))
        .route("/recommendations/:id", get(get_by_id))
        .route(
            "/recommendations/severity/:percentage",
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
async fn get_all(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let recommendations = state.recommendation_service.get_all().await?;

    if recommendations.is_empty() {
        Err(AppError::NoContent(
            "There are no recommendations available".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_iter_json::<_, SimplifiedRecommendationResponse, RecommendationResponse>(
                recommendations,
                query.simplified,
            ),
        ))
    }
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
async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let result = state.recommendation_service.get_by_id(id).await?;

    if result.is_none() {
        Err(AppError::NotFound(
            "The recommendation with the specified ID does not exist".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_json::<_, SimplifiedRecommendationResponse, RecommendationResponse>(
                result.unwrap(),
                query.simplified,
            ),
        ))
    }
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

    Ok((
        StatusCode::OK,
        conditional_iter_json::<_, SimplifiedRecommendationResponse, RecommendationResponse>(
            recommendations,
            query.simplified,
        ),
    ))
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
async fn create(
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
        (status = 200, description = "Recommendation updated", body = RecommendationResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 404, description = "Not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendations"
)]
async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateRecommendationRequest>,
) -> Result<impl IntoResponse> {
    let result = state
        .recommendation_service
        .update(id, payload.into())
        .await?;

    Ok((StatusCode::OK, Json(RecommendationResponse::from(result))))
}

#[utoipa::path(
    delete,
    path = "/recommendations/{id}",
    params(
        ("id" = Uuid, Path, description = "Recommendation ID")
    ),
    responses(
        (status = 200, description = "Recommendation deleted"),
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
    let result = state.recommendation_service.delete(id).await?;

    Ok((StatusCode::OK, Json(RecommendationResponse::from(result))))
}
