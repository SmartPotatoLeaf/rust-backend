use crate::adapters::web::state::AppState;
use crate::adapters::web::{
    middleware::{
        auth::AuthUser,
        permissions::{permission_check, RequiredRoles, RoleValidation},
    },
    models::{
        common::SimplifiedQuery,
        recommendation::{
            CreateRecommendationCategoryRequest, RecommendationCategoryResponse,
            SimplifiedRecommendationCategoryResponse, UpdateRecommendationCategoryRequest,
        },
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{ok_if_or_not_found, ok_iter_if_or_not_found, StatusResponse};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum RecommendationCategoryOrSimplifiedResponse {
    Category(RecommendationCategoryResponse),
    Simplified(SimplifiedRecommendationCategoryResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(get_all_recommendation_categories, get_recommendation_category_by_id, create_recommendation_category, update_recommendation_category, delete_recommendation_category),
    components(schemas(
        CreateRecommendationCategoryRequest,
        UpdateRecommendationCategoryRequest,
        RecommendationCategoryResponse,
        SimplifiedRecommendationCategoryResponse,
        RecommendationCategoryOrSimplifiedResponse,
        StatusResponse
    )),
    tags((name = "recommendation_categories", description = "Recommendation Category endpoints")),
    security(("jwt_auth" = []))
)]
pub struct CategoryApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let admin_only_layer = middleware::from_fn_with_state(state.clone(), permission_check);
    let admin_extension_roles = Extension(RequiredRoles(
        vec!["admin".to_string()],
        RoleValidation::Higher,
    ));

    let admin_router = Router::new()
        .route("/recommendation/categories", post(create_recommendation_category))
        .route(
            "/recommendation/categories/{id}",
            delete(delete_recommendation_category).put(update_recommendation_category),
        )
        .route_layer(admin_only_layer)
        .route_layer(admin_extension_roles)
        .with_state(state.clone());

    let public_router = Router::new()
        .route("/recommendation/categories", get(get_all_recommendation_categories))
        .route("/recommendation/categories/{id}", get(get_recommendation_category_by_id))
        .with_state(state);

    Router::new().merge(public_router).merge(admin_router)
}

#[utoipa::path(
    get,
    path = "/recommendation/categories",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "List all recommendation categories", body = Vec<RecommendationCategoryOrSimplifiedResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No recommendation categories found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendation_categories"
)]
async fn get_all_recommendation_categories(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let types = state.recommendation_category_service.get_all().await?;

    ok_iter_if_or_not_found(
        types,
        query.simplified,
        SimplifiedRecommendationCategoryResponse::from,
        RecommendationCategoryResponse::from,
        || "There are no recommendation categories available".to_string(),
    )
}

#[utoipa::path(
    get,
    path = "/recommendation/categories/{id}",
    params(
        ("id" = i32, Path, description = "Recommendation Category ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Recommendation Category found", body = RecommendationCategoryOrSimplifiedResponse),
        (status = 404, description = "Recommendation Category not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendation_categories"
)]
async fn get_recommendation_category_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Query(query): Query<SimplifiedQuery>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let result = state.recommendation_category_service.get_by_id(id).await?;

    ok_if_or_not_found(
        result,
        query.simplified,
        SimplifiedRecommendationCategoryResponse::from,
        RecommendationCategoryResponse::from,
        move || format!("The recommendation category with id {} does not exist", id),
    )
}

#[utoipa::path(
    post,
    path = "/recommendation/categories",
    request_body = CreateRecommendationCategoryRequest,
    responses(
        (status = 201, description = "Recommendation Category created", body = RecommendationCategoryResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendation_categories"
)]
async fn create_recommendation_category(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateRecommendationCategoryRequest>,
) -> Result<impl IntoResponse> {
    let result = state
        .recommendation_category_service
        .create(payload.into())
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(RecommendationCategoryResponse::from(result)),
    ))
}

#[utoipa::path(
    put,
    path = "/recommendation/categories/{id}",
    params(
        ("id" = i32, Path, description = "Recommendation Category ID")
    ),
    request_body = UpdateRecommendationCategoryRequest,
    responses(
        (status = 200, description = "Recommendation Category updated", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 404, description = "Not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendation_categories"
)]
async fn update_recommendation_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    _user: AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateRecommendationCategoryRequest>,
) -> Result<impl IntoResponse> {
    let _ = state
        .recommendation_category_service
        .update(id, payload.into())
        .await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Recommendation Category updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/recommendation/categories/{id}",
    params(
        ("id" = i32, Path, description = "Recommendation Category ID")
    ),
    responses(
        (status = 200, description = "Recommendation Category deleted", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden (Admin only)", body = StatusResponse),
        (status = 404, description = "Not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "recommendation_categories"
)]
async fn delete_recommendation_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    _user: AuthUser,
) -> Result<impl IntoResponse> {
    let _ = state.recommendation_category_service.delete(id).await?;

    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Recommendation Category deleted successfully".to_string(),
        }),
    ))
}
