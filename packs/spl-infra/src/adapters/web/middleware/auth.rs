use crate::adapters::web::state::AppState;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
    response::{IntoResponse, Response},
};
use spl_domain::entities::user::User;
use spl_shared::error::AppError;
use std::sync::Arc;

pub struct AuthUser(pub User);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AppError::AuthError("Missing authorization header".to_string()).into_response()
            })?;

        if !auth_header.starts_with("Bearer ") {
            return Err(
                AppError::AuthError("Invalid authorization header format".to_string())
                    .into_response(),
            );
        }

        let token = &auth_header[7..];

        let claims = state
            .auth_service
            .validate_token(token)
            .map_err(|_| AppError::AuthError("Invalid token".to_string()).into_response())?;

        // Extract user_id from sub
        let user_id_str = claims["sub"].as_str().ok_or_else(|| {
            AppError::AuthError("Invalid token claims".to_string()).into_response()
        })?;

        let user_id = uuid::Uuid::parse_str(user_id_str).map_err(|_| {
            AppError::AuthError("Invalid user id in token".to_string()).into_response()
        })?;

        // Get user from DB
        let user = state
            .user_service
            .get_by_id(user_id)
            .await
            .map_err(|e| {
                e.into_response()
            })?
            .ok_or_else(|| {
                AppError::NotFound(format!("User with id {user_id} not found").to_string())
                    .into_response()
            })?;

        Ok(AuthUser(user))
    }
}
