use crate::adapters::web::mappers::diagnostics::prediction::FilterPredictionMapperContext;
use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::models::image::ImageResponse;
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
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use spl_shared::error::AppError;
use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{conditional_iter_json, ok_or_not_found, StatusResponse};
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
    paths(create, get_all_by_user_id, filter, get_by_id, delete_prediction, read_blob),
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
        StatusResponse
    )),
    tags((name = "diagnostics/predictions", description = "Prediction management endpoints"))
)]
pub struct PredictionApi;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/diagnostics/predictions",
            post(create).get(get_all_by_user_id),
        )
        .route("/diagnostics/predictions/filter", post(filter))
        .route(
            "/diagnostics/predictions/:id",
            get(get_by_id).delete(delete_prediction),
        )
        .route("/diagnostics/predictions/blobs/*path", get(read_blob))
        .with_state(state)
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
    let mut file_bytes = None;
    let mut filename: String = Uuid::new_v4().to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::ValidationError(format!("Failed to process multipart: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            if let Some(fname) = field.file_name() {
                filename = fname.to_string();
            }

            let data = field.bytes().await.map_err(|e| {
                AppError::ValidationError(format!("Failed to read file bytes: {}", e))
            })?;
            file_bytes = Some(data.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| AppError::ValidationError("No file provided".into()))?;

    let prediction = state
        .prediction_service
        .predict_and_create(user.id, bytes, filename)
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

    if predictions.is_empty() {
        Err(AppError::NoContent(
            "No predictions found for this user".to_string(),
        ))
    } else {
        Ok((
            StatusCode::OK,
            conditional_iter_json::<_, SimplifiedPredictionResponse, PredictionResponse>(
                predictions,
                query.simplified,
            ),
        ))
    }
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

    let response = result.map(|p| {
        if query.simplified {
            PredictionOrSimplifiedResponse::Simplified(p.into())
        } else {
            PredictionOrSimplifiedResponse::Prediction(p.into())
        }
    });

    ok_or_not_found(response, format!("Prediction {} not found", id))
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
    path = "/diagnostics/predictions/blobs/*path",
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
