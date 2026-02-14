use crate::setup::database::initialize_database;
use crate::setup::integrations;
use crate::setup::integrations::{initialize_model_client, initialize_storage_client};
use crate::setup::rate_limiting::initialize_rate_limiting;
use crate::setup::redis::initialize_redis;
use crate::setup::repositories::{initialize_adapters, initialize_repositories};
use crate::setup::seed::{load_role_cache, seed_admin_user};
use crate::setup::services::initialize_services;
use anyhow::Result;
use spl_infra::adapters::web::{router, state::AppState};
use spl_shared::config::AppConfig;
use spl_shared::telemetry::init_telemetry;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

mod setup;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load environment variables
    dotenvy::dotenv().ok();

    // 2. Initialize Telemetry
    init_telemetry();
    info!("Starting SmartPotatoLeaf Backend...");

    // 3. Load Configuration
    let config = Arc::new(AppConfig::load()?);
    info!(
        "Configuration loaded from: {}:{}",
        config.server.host, config.server.port
    );

    // 4. Initialize Database
    let db = initialize_database(&config.database).await?;

    // 5. Initialize Repositories & Adapters
    let repos = initialize_repositories(db);
    let adapters = initialize_adapters(config.clone());

    // 5.1 Seed Admin User
    if let Some(admin_config) = &config.admin {
        seed_admin_user(
            admin_config,
            &repos.role_repo,
            &repos.user_repo,
            &adapters.password_encoder,
        )
        .await?;
    }

    // 6. Initialize Integration Clients
    info!("Initializing integration clients...");
    let model_client = initialize_model_client(&config.integrations).await?;
    let storage_client = initialize_storage_client(&config.integrations).await?;

    // 6.1 Health Checks
    integrations::health_checks(&model_client, &storage_client).await?;

    // 6.2 Initialize Redis (shared infrastructure for rate limiting, caching, etc.)
    let redis_pool = initialize_redis(&config.redis).await;

    // 6.3 Initialize Rate Limiting
    let rate_limit_state = initialize_rate_limiting(&config, redis_pool);

    // 7. Initialize Services
    let services = initialize_services(
        &repos,
        &adapters,
        model_client.clone(),
        storage_client.clone(),
    );

    // 8. Load Role Cache
    let role_cache = load_role_cache(&repos.role_repo).await?;

    // 9. Initialize Web Router & State
    let app_state = Arc::new(AppState::new(
        config.clone(),
        services.auth_service,
        services.role_service,
        services.user_service,
        services.company_service,
        services.image_service,
        services.recommendation_category_service,
        services.recommendation_service,
        services.label_service,
        services.mark_type_service,
        services.prediction_service,
        services.plot_service,
        services.dashboard_service,
        services.feedback_service,
        services.feedback_status_service,
        role_cache,
        model_client,
        storage_client,
    ));

    let app = router(app_state, rate_limit_state);

    // 10. Start Server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
