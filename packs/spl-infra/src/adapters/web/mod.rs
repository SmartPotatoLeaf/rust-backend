use crate::adapters::web::controllers::{auth, companies, dashboard, diagnostics, feedback, plots, recommendation, user};
use crate::adapters::web::state::AppState;
use axum::http::HeaderValue;
use axum::Router;
use http::{header, Method};
use spl_shared::config::AppConfig;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::info;
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

fn build_cors_layer(config: &AppConfig) -> Option<CorsLayer> {
    let origins_raw = config.server.cors_allowed_origins.as_ref()?;
    let origins: Vec<String> = origins_raw
        .split(',')
        .map(|origin| origin.trim())
        .filter(|origin| !origin.is_empty())
        .map(|origin| origin.to_string())
        .collect();

    if origins.is_empty() {
        return None;
    }

    if origins.iter().any(|origin| origin == "*") {
        return Some(CorsLayer::new().allow_origin(Any));
    }

    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();

    if allowed.is_empty() {
        return None;
    }

    Some(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE])
            .expose_headers([header::AUTHORIZATION])
            .allow_credentials(true),
    )
}

pub fn router(state: Arc<AppState>) -> Router {
    let mut openapi = ApiDoc::openapi();
    let base_path = "/api/v1";

    openapi.merge(auth::AuthApi::openapi());
    openapi.merge(user::UserApi::openapi());
    openapi.merge(companies::CompaniesApi::openapi());
    openapi.merge(dashboard::DashboardApi::openapi());
    openapi.merge(recommendation::CategoryApi::openapi());
    openapi.merge(recommendation::RecommendationApi::openapi());
    openapi.merge(plots::PlotsApi::openapi());
    openapi.merge(diagnostics::labels::LabelsApi::openapi());
    openapi.merge(diagnostics::mark_types::MarkTypesApi::openapi());
    openapi.merge(diagnostics::prediction::PredictionApi::openapi());
    openapi.merge(feedback::status::FeedbackStatusApi::openapi());
    openapi.merge(feedback::FeedbackApi::openapi());

    let addon = SecurityAddon {};
    addon.modify(&mut openapi);

    let mut app = Router::new()
        .merge(
            SwaggerUi::new(base_path.to_string() + "/swagger-ui")
                .url("/api-docs/openapi.json", openapi),
        )
        .nest(base_path, auth::router())
        .nest(base_path, user::router(state.clone()))
        .nest(base_path, companies::router(state.clone()))
        .nest(base_path, dashboard::router(state.clone()))
        .nest(base_path, recommendation::category::router(state.clone()))
        .nest(base_path, recommendation::router(state.clone()))
        .nest(base_path, diagnostics::labels::router(state.clone()))
        .nest(base_path, diagnostics::mark_types::router(state.clone()))
        .nest(base_path, diagnostics::prediction::router(state.clone()))
        .nest(base_path, plots::router(state.clone()))
        .nest(base_path, feedback::status::router(state.clone()))
        .nest(base_path, feedback::router());

    if let Some(cors_layer) = build_cors_layer(state.config.as_ref()) {
        info!("CORS layer enabled");
        app = app.layer(cors_layer);
    } else {
        info!("CORS layer not enabled");
    }

    app.with_state(state)
}
