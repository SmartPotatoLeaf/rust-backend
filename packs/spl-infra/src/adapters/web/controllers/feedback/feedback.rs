use crate::adapters::web::middleware::auth::AuthUser;
use crate::adapters::web::models::common::SimplifiedQuery;
use crate::adapters::web::models::feedback::feedback::{
    CreateFeedbackRequest, FeedbackResponse, SimplifiedFeedbackResponse, UpdateFeedbackRequest,
};
use crate::adapters::web::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use spl_shared::error::Result;
use spl_shared::http::extractor::ValidatedJson;
use spl_shared::http::responses::{ok_if_or_not_found, ok_iter_if_or_not_found, StatusResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(OpenApi)]
#[openapi(
    paths(
        create,
        get_all_by_user,
        get_by_id,
        get_by_prediction,
        update,
        delete_feedback
    ),
    components(schemas(
        CreateFeedbackRequest,
        UpdateFeedbackRequest,
        FeedbackResponse,
        SimplifiedFeedbackResponse,
        StatusResponse
    )),
    tags((name = "feedbacks", description = "Feedback endpoints")),
    security(("jwt_auth" = []))
)]
pub struct FeedbackApi;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/feedbacks", get(get_all_by_user).post(create))
        .route("/feedbacks/:id", get(get_by_id).put(update).delete(delete_feedback))
        .route("/feedbacks/prediction/:prediction_id", get(get_by_prediction))
}

#[utoipa::path(
    post,
    path = "/feedbacks",
    request_body = CreateFeedbackRequest,
    responses(
        (status = 201, description = "Feedback created successfully", body = FeedbackResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 409, description = "Feedback already exists for this prediction", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedbacks"
)]
async fn create(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<CreateFeedbackRequest>,
) -> Result<impl IntoResponse> {
    let created = state
        .feedback_service
        .create_by_user(&user, payload.into())
        .await?;
    Ok((StatusCode::CREATED, Json(FeedbackResponse::from(created))))
}

#[utoipa::path(
    get,
    path = "/feedbacks",
    params(
        ("simplified" = Option<bool>, Query, description = "Return simplified response")
    ),
    responses(
        (status = 200, description = "List all feedbacks for current user", body = Vec<FeedbackResponse>),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedbacks"
)]
async fn get_all_by_user(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let feedbacks = state.feedback_service.get_all_by_user(&user).await?;

    ok_iter_if_or_not_found(
        feedbacks,
        query.simplified,
        SimplifiedFeedbackResponse::from,
        FeedbackResponse::from,
        || "There are no feedbacks available".to_string(),
    )
}

#[utoipa::path(
    get,
    path = "/feedbacks/{id}",
    params(
        ("id" = Uuid, Path, description = "Feedback ID"),
        ("simplified" = Option<bool>, Query, description = "Return simplified response")
    ),
    responses(
        (status = 200, description = "Feedback found", body = FeedbackResponse),
        (status = 404, description = "Feedback not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedbacks"
)]
async fn get_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state.feedback_service.get_by_id_and_user(id, &user).await?;

    ok_if_or_not_found(
        result,
        query.simplified,
        SimplifiedFeedbackResponse::from,
        FeedbackResponse::from,
        move || format!("The feedback with id {} does not exist", id),
    )
}

#[utoipa::path(
    get,
    path = "/feedbacks/prediction/{prediction_id}",
    params(
        ("prediction_id" = Uuid, Path, description = "Prediction ID"),
        ("simplified" = Option<bool>, Query, description = "Return simplified response")
    ),
    responses(
        (status = 200, description = "Feedback found", body = FeedbackResponse),
        (status = 404, description = "Feedback not found", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedbacks"
)]
async fn get_by_prediction(
    State(state): State<Arc<AppState>>,
    Path(prediction_id): Path<Uuid>,
    AuthUser(user): AuthUser,
    Query(query): Query<SimplifiedQuery>,
) -> Result<impl IntoResponse> {
    let result = state
        .feedback_service
        .get_by_user_and_prediction(&user, prediction_id)
        .await?;

    ok_if_or_not_found(
        result,
        query.simplified,
        SimplifiedFeedbackResponse::from,
        FeedbackResponse::from,
        move || format!("The feedback for prediction {} does not exist", prediction_id),
    )
}

#[utoipa::path(
    put,
    path = "/feedbacks/{id}",
    params(
        ("id" = Uuid, Path, description = "Feedback ID")
    ),
    request_body = UpdateFeedbackRequest,
    responses(
        (status = 200, description = "Feedback updated successfully", body = StatusResponse),
        (status = 400, description = "Invalid input", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Feedback not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedbacks"
)]
async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
    ValidatedJson(payload): ValidatedJson<UpdateFeedbackRequest>,
) -> Result<impl IntoResponse> {
    let _ = state
        .feedback_service
        .update_by_user(id, &user, payload.into())
        .await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Feedback updated successfully".to_string(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/feedbacks/{id}",
    params(
        ("id" = Uuid, Path, description = "Feedback ID")
    ),
    responses(
        (status = 200, description = "Feedback deleted successfully", body = StatusResponse),
        (status = 401, description = "Unauthorized", body = StatusResponse),
        (status = 404, description = "Feedback not found", body = StatusResponse),
        (status = 500, description = "Internal Server Error", body = StatusResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "feedbacks"
)]
async fn delete_feedback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> Result<impl IntoResponse> {
    let _ = state.feedback_service.delete_by_user(id, &user).await?;
    Ok((
        StatusCode::OK,
        Json(StatusResponse {
            success: true,
            code: 200,
            message: "Feedback deleted successfully".to_string(),
        }),
    ))
}
