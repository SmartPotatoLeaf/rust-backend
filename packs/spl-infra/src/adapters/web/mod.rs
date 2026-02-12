use crate::adapters::web::controllers::{
    auth, companies, diagnostics, plots, recommendation, user,
};
use crate::adapters::web::state::AppState;
use axum::Router;
use std::sync::Arc;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

pub mod controllers;
pub mod mappers;
pub mod middleware;
pub mod models;
pub mod state;

#[derive(OpenApi)]
#[openapi(
    servers((url = "/api/v1")),
    info(
        title = "SPL Backend API",
        description = "API documentation for the SPL Backend service.",
        version = "1.0.0"
    ),
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

pub fn router(state: Arc<AppState>) -> Router {
    let mut openapi = ApiDoc::openapi();
    let base_path = "/api/v1";

    openapi.merge(auth::AuthApi::openapi());
    openapi.merge(user::UserApi::openapi());
    openapi.merge(companies::CompaniesApi::openapi());
    openapi.merge(recommendation::CategoryApi::openapi());
    openapi.merge(recommendation::RecommendationApi::openapi());
    openapi.merge(plots::PlotsApi::openapi());
    openapi.merge(diagnostics::labels::LabelsApi::openapi());
    openapi.merge(diagnostics::mark_types::MarkTypesApi::openapi());
    openapi.merge(diagnostics::prediction::PredictionApi::openapi());

    let addon = SecurityAddon {};
    addon.modify(&mut openapi);

    Router::new()
        .merge(
            SwaggerUi::new(base_path.to_string() + "/swagger-ui")
                .url("/api-docs/openapi.json", openapi),
        )
        .nest(base_path, auth::router())
        .nest(base_path, user::router())
        .nest(base_path, companies::router(state.clone()))
        .nest(base_path, recommendation::category::router(state.clone()))
        .nest(base_path, recommendation::router(state.clone()))
        .nest(base_path, diagnostics::labels::router(state.clone()))
        .nest(base_path, diagnostics::mark_types::router(state.clone()))
        .nest(base_path, diagnostics::prediction::router(state.clone()))
        .nest(base_path, plots::router(state.clone()))
        .with_state(state)
}
