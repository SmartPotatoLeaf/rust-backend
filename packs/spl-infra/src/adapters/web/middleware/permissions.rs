use crate::adapters::web::{middleware::auth::AuthUser, state::AppState};
use axum::response::IntoResponse;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    Extension,
};
use spl_shared::error::AppError;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleValidation {
    Higher,     // User level >= Required level
    Lower,      // User level <= Required level
    Same,       // User level == Required level
    SameStrict, // User role name == Required role name AND value matches
}

#[derive(Clone, Debug)]
pub struct RequiredRoles(pub Vec<String>, pub RoleValidation);

pub async fn permission_check(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Extension(required_roles): Extension<RequiredRoles>,
    request: Request,
    next: Next,
) -> Response {
    // 1. Fetch User's Role Level from Cache
    let user_role_name = &user.role.name; // Extracted from JWT
    let user_level = state.roles.get(user_role_name).copied().unwrap_or(0);

    // Special case for SameStrict: we check if user has EXACTLY one of the role names
    if required_roles.1 == RoleValidation::SameStrict {
        return if required_roles.0.contains(user_role_name)
            && state.roles.get(user_role_name).copied().unwrap_or(i16::MIN) == user_level
        {
            next.run(request).await
        } else {
            AppError::Forbidden.into_response()
        };
    }

    match required_roles.1 {
        RoleValidation::Same => {
            if required_roles
                .0
                .iter()
                .any(|r| state.roles.get(r).copied().unwrap_or(i16::MIN) == user_level)
            {
                return next.run(request).await;
            }
        }
        RoleValidation::Higher => {
            let max = required_roles
                .0
                .iter()
                .filter_map(|r| state.roles.get(r))
                .min()
                .copied()
                .unwrap_or(i16::MIN);

            if user_level >= max {
                return next.run(request).await;
            }
        }
        RoleValidation::Lower => {
            let max = required_roles
                .0
                .iter()
                .filter_map(|r| state.roles.get(r))
                .max()
                .copied()
                .unwrap_or(i16::MAX);
            if user_level <= max {
                return next.run(request).await;
            }
        }
        RoleValidation::SameStrict => {} // Handled above
    }

    AppError::Forbidden.into_response()
}
