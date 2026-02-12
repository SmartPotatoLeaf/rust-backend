use crate::error::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::error;

use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Deserialize)]
pub struct StatusResponse {
    pub success: bool,
    pub code: u16,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Unauthorized access".to_string(),
            ),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, "NOT_FOUND", message),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            AppError::AuthError(message) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", message),
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "AUTH_ERROR",
                "Invalid username or password".to_string(),
            ),
            AppError::UserAlreadyExists => (
                StatusCode::CONFLICT,
                "USER_ALREADY_EXISTS",
                "User already exists inside this company".to_string(),
            ),
            AppError::IntegrationError {
                integration,
                message,
            } => {
                error!("Integration error in {}: {:?}", integration, message);
                (
                    StatusCode::BAD_GATEWAY,
                    "INTEGRATION_ERROR",
                    format!("External service error: {}", message),
                )
            }
            AppError::IntegrationTimeout(message) => {
                error!("Integration timeout: {:?}", message);
                (StatusCode::GATEWAY_TIMEOUT, "INTEGRATION_TIMEOUT", message)
            }
            AppError::IntegrationUnavailable(message) => {
                error!("Integration unavailable: {:?}", message);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "INTEGRATION_UNAVAILABLE",
                    message,
                )
            }
            AppError::Unknown(err) => {
                error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Unexpected error".to_string(),
                )
            }
            AppError::DatabaseError(message) => {
                error!("{:?}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Database error occurred".to_string(),
                )
            }
            AppError::ConfigError(message) => {
                error!("{:?}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Configuration error occurred".to_string(),
                )
            }
            AppError::NoContent(message) => {
                error!("{:?}", message);
                (StatusCode::NOT_FOUND, "NO_CONTENT", message)
            }
        };

        (
            status,
            Json(StatusResponse {
                success: false,
                code: status.as_u16(),
                message: format!("{}: {}", code, message),
            }),
        )
            .into_response()
    }
}

pub fn json_if<T, F1, F2, R1, R2>(
    data: T,
    condition: bool,
    on_true: F1,
    on_false: F2,
) -> impl IntoResponse
where
    F1: FnOnce(T) -> R1,
    F2: FnOnce(T) -> R2,
    R1: Serialize,
    R2: Serialize,
{
    if condition {
        Json(on_true(data)).into_response()
    } else {
        Json(on_false(data)).into_response()
    }
}

pub fn iter_json_if<T, F1, F2, R1, R2>(
    data: impl IntoIterator<Item = T>,
    condition: bool,
    on_true: F1,
    on_false: F2,
) -> impl IntoResponse
where
    F1: FnMut(T) -> R1,
    F2: FnMut(T) -> R2,
    R1: Serialize,
    R2: Serialize,
{
    if condition {
        Json(data.into_iter().map(on_true).collect::<Vec<_>>()).into_response()
    } else {
        Json(data.into_iter().map(on_false).collect::<Vec<_>>()).into_response()
    }
}

pub fn ok_or_not_found<T>(
    option: Option<T>,
    not_found_msg: String,
) -> Result<impl IntoResponse, AppError>
where
    T: Serialize,
{
    match option {
        Some(data) => Ok((StatusCode::OK, Json(data))),
        None => Err(AppError::NotFound(not_found_msg)),
    }
}

pub fn ok_if_or_not_found<T, F1, F2, F3, R1, R2>(
    data: Option<T>,
    condition: bool,
    on_true: F1,
    on_false: F2,
    not_found: F3,
) -> Result<impl IntoResponse, AppError>
where
    F1: FnOnce(T) -> R1,
    F2: FnOnce(T) -> R2,
    F3: FnOnce() -> String,
    R1: Serialize,
    R2: Serialize,
{
    match data {
        Some(data) => Ok((StatusCode::OK, json_if(data, condition, on_true, on_false))),
        None => Err(AppError::NotFound(not_found())),
    }
}

pub fn ok_iter_if_or_not_found<T, F1, F2, F3, R1, R2>(
    data: Vec<T>,
    condition: bool,
    on_true: F1,
    on_false: F2,
    no_content: F3,
) -> Result<impl IntoResponse, AppError>
where
    F1: FnMut(T) -> R1,
    F2: FnMut(T) -> R2,
    F3: FnOnce() -> String,
    R1: Serialize,
    R2: Serialize,
{
    if data.is_empty() {
        Err(AppError::NoContent(no_content()))
    } else {
        Ok((StatusCode::OK, iter_json_if(data, condition, on_true, on_false)))
    }
}