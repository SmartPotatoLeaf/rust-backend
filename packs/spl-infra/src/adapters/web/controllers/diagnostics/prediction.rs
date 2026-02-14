use crate::adapters::web::mappers::diagnostics::prediction::FilterPredictionMapperContext;
use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::models::diagnostics::label::RawLabelResponse;
use crate::adapters::web::models::diagnostics::prediction_mark::RawPredictionMarkResponse;
use crate::adapters::web::models::diagnostics::RawPredictionResponse;
use crate::adapters::web::models::image::{ImageResponse, RawImageResponse};
use crate::adapters::web::models::{
    common::SimplifiedQuery,
    diagnostics::{
        prediction::{
            CreatePredictionRequest, FilterPredictionsRequest, PredictionResponse,
            PredictionsListResponse, SimplifiedPredictionResponse,
        },
        prediction_mark::PredictionMarkResponse,
        LabelResponse, MarkTypeResponse,
    },
};
use crate::adapters::web::state::AppState;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use spl_shared::error::AppError;
use spl_shared::error::Result;
use spl_shared::http::extractor::multipart::extract_file;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::middleware::{
    local_rate_limit_middleware, EndpointRateLimit, RateLimitState,
};
use spl_shared::http::responses::{ok_if_or_not_found, ok_iter_if_or_not_found, StatusResponse};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
#[serde(untagged)]
enum PredictionOrSimplifiedResponse {
    Prediction(PredictionResponse),
    Simplified(SimplifiedPredictionResponse),
}

#[derive(OpenApi)]
#[openapi(
    paths(create, get_all_by_user_id, filter, get_by_id, delete_prediction, read_blob, predict),
    components(schemas(
        CreatePredictionRequest,
        PredictionResponse,
        PredictionMarkResponse,
        FilterPredictionsRequest,
        PredictionsListResponse,
        ImageResponse,
        LabelResponse,
        MarkTypeResponse,
        SimplifiedPredictionResponse,
        PredictionOrSimplifiedResponse,
        StatusResponse,
        RawPredictionResponse,
        RawImageResponse,
        RawLabelResponse,
        RawPredictionMarkResponse
    )),
    tags((name = "diagnostics/predictions", description = "Prediction management endpoints"))
)]
pub struct PredictionApi;

pub fn router(state: Arc<AppState>, limit_state: Arc<RateLimitState>) -> Router<Arc<AppState>> {
    let mut router = Router::new();

    if let Some(config) = state.config.rate_limiting.clone() {
        let rate_limit_layer = middleware::from_fn_with_state(
            limit_state.clone(),
            local_rate_limit_middleware,
        );

        router = router.route(
            "/public/diagnostics/predict",
            post(predict)
                .route_layer(rate_limit_layer)
                .route_layer(Extension(
                    EndpointRateLimit::new(config.default_limit)
                        .with_window(config.window_seconds)
                        .with_behavior(
                            config.endpoint_behavior.map(Into::into).unwrap_or_default(),
                        ),
                )),
        )
    }

    router
        .route(
            "/diagnostics/predictions",
            post(create).get(get_all_by_user_id),
        )
        .route("/diagnostics/predictions/filter", post(filter))
        .route(
            "/diagnostics/predictions/{id}",
            get(get_by_id).delete(delete_prediction),
        )
        .route("/diagnostics/predictions/blobs/{*path}", get(read_blob))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/public/diagnostics/predict",
    request_body(content = CreatePredictionRequest, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Prediction realized", body = RawPredictionResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 429, description = "Too Many Requests", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    tag = "diagnostics/predictions"
)]
async fn predict(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let (bytes, filename) = extract_file("file", &mut multipart).await?;
    let prediction = state
        .prediction_service
        .predict(bytes, filename.unwrap_or(Uuid::new_v4().to_string()))
        .await?;

    Ok((
        StatusCode::OK,
        Json(RawPredictionResponse::from(prediction)),
    ))
}

#[utoipa::path(
    post,
    path = "/diagnostics/predictions",
    request_body(content = CreatePredictionRequest, content_type = "multipart/form-data"),
    responses(
        (status = 201, description = "Prediction created", body = PredictionResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/predictions"
)]
async fn create(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse> {
    let (bytes, filename) = extract_file("file", &mut multipart).await?;

    let prediction = state
        .prediction_service
        .predict_and_create(
            user.id,
            bytes,
            filename.unwrap_or(Uuid::new_v4().to_string()),
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(PredictionResponse::from(prediction)),
    ))
}

#[utoipa::path(
    get,
    path = "/diagnostics/predictions",
    params(
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "List valid predictions", body = Vec<PredictionOrSimplifiedResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "No predictions found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/predictions"
)]
async fn get_all_by_user_id(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SimplifiedQuery>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let predictions = state.prediction_service.get_by_user_id(user.id).await?;

    ok_iter_if_or_not_found(
        predictions,
        query.simplified,
        SimplifiedPredictionResponse::from,
        PredictionResponse::from,
        || "No predictions found for this user".to_string(),
    )
}

#[utoipa::path(
    get,
    path = "/diagnostics/predictions/{id}",
    params(
        ("id" = Uuid, Path, description = "Prediction ID"),
        SimplifiedQuery
    ),
    responses(
        (status = 200, description = "Prediction details", body = PredictionOrSimplifiedResponse),
        (status = 404, description = "Prediction not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/predictions"
)]
async fn get_by_id(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state
        .prediction_service
        .get_by_user_id_and_id(user.id, id)
        .await?;

    ok_if_or_not_found(
        result,
        query.simplified,
        SimplifiedPredictionResponse::from,
        PredictionResponse::from,
        || "Prediction not found".to_string(),
    )
}

#[utoipa::path(
    delete,
    path = "/diagnostics/predictions/{id}",
    params(("id" = Uuid, Path, description = "Prediction ID")),
    responses(
        (status = 200, description = "Prediction deleted", body = StatusResponse),
        (status = 404, description = "Prediction not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/predictions"
)]
async fn delete_prediction(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let _ = state.prediction_service.delete(user.id, id).await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Prediction deleted successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/diagnostics/predictions/filter",
    request_body = FilterPredictionsRequest,
    responses(
        (status = 200, description = "Filtered predictions", body = PredictionsListResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/predictions"
)]
async fn filter(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<FilterPredictionsRequest>,
) -> Result<impl IntoResponse> {
    let context = FilterPredictionMapperContext {
        requester: user.clone(),
    };

    let dto = payload.clone().into_with_context(context)?;

    let (total, items) = state.prediction_service.filter(dto, &user).await?;

    let limit = payload.limit.unwrap_or(16);
    let page = payload.page.unwrap_or(1);

    Ok((
        StatusCode::OK,
        Json(PredictionsListResponse {
            total,
            page,
            limit,
            items: items.into_iter().map(|p| p.into()).collect(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/diagnostics/predictions/blobs/{*path}",
    params(("path" = String, Path, description = "Blob path")),
    responses(
        (status = 200, description = "Blob content"),
        (status = 404, description = "Blob not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 403, description = "Forbidden", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "diagnostics/predictions"
)]
async fn read_blob(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(path): Path<String>,
) -> Result<impl IntoResponse> {
    let prefix = format!("{}/", user.id);

    if !path.starts_with(&prefix) {
        return Err(AppError::Forbidden);
    }

    let bytes = state
        .storage_client
        .download(&path)
        .await
        .map_err(|e| AppError::NotFound(format!("Blob not found: {}", e)))?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "image/jpeg".parse().unwrap());

    Ok((StatusCode::OK, headers, bytes).into_response())
}
