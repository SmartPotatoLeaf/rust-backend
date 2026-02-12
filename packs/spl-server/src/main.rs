use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use spl_application::{
    services,
    services::{
        auth::AuthService,
        company::CompanyService,
        diagnostics::{LabelService, MarkTypeService},
        image::ImageService,
        plot::PlotService,
        recommendation::RecommendationService,
        user::{role::RoleService, UserService},
    },
};
use spl_domain::entities::user::User;
use spl_domain::ports::auth::PasswordEncoder;
use spl_domain::ports::integrations::{BlobStorageClient, ModelPredictionClient};
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::user::{RoleRepository, UserRepository};
use spl_infra::adapters::integrations::{
    model_serving::{
        mock::MockModelClient,
        tensorflow::{TensorFlowServingClient, TensorFlowServingGrpcClient},
    },
    storage::{azure::AzureBlobClient, local::LocalFileSystemClient, mock::MockBlobClient},
};
use spl_infra::adapters::{
    auth::{jwt::JwtTokenGenerator, password::Argon2PasswordEncoder},
    persistence::{
        repositories,
        repositories::{
            company::DbCompanyRepository,
            diagnostics::{
                DbLabelRepository, DbMarkTypeRepository, DbPredictionMarkRepository,
                DbPredictionRepository,
            },
            image::DbImageRepository,
            plot::DbPlotRepository,
            recommendation::DbRecommendationRepository,
            user::{role::DbRoleRepository, DbUserRepository},
        },
    },
    web::{router, state::AppState},
};
use spl_shared::config::AppConfig;
use spl_shared::telemetry::init_telemetry;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

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
    let mut opt = ConnectOptions::new(config.database.url.clone());
    opt.max_connections(config.database.max_connections.unwrap_or(20))
        .min_connections(config.database.min_connections.unwrap_or(5))
        .connect_timeout(Duration::from_secs(
            config.database.connect_timeout.unwrap_or(10),
        ))
        .idle_timeout(Duration::from_secs(
            config.database.idle_timeout.unwrap_or(300),
        ))
        .max_lifetime(Duration::from_secs(
            config.database.max_lifetime.unwrap_or(1800),
        ));

    let db: DatabaseConnection = Database::connect(opt).await?;
    info!("Connected to database: {}", config.database.url);

    // 4.1 Run Migrations
    use sea_orm_migration::MigratorTrait;
    use spl_migration::Migrator;
    Migrator::up(&db, None).await?;
    info!("Migrations executed successfully.");

    // 5. Initialize Adapters
    let role_repo = Arc::new(DbRoleRepository::new(db.clone()));
    let company_repo = Arc::new(DbCompanyRepository::new(db.clone()));
    let user_repo = Arc::new(DbUserRepository::new(
        db.clone(),
        role_repo.clone(),
        company_repo.clone(),
    ));
    let password_encoder = Arc::new(Argon2PasswordEncoder::new());
    let token_generator = Arc::new(JwtTokenGenerator::new(config.clone()));

    // 5.1 Seed Admin User
    if let Some(admin_config) = &config.admin {
        info!("Checking for Admin user seeding...");
        match role_repo.get_by_name("admin").await {
            Ok(Some(admin_role)) => {
                match user_repo
                    .get_by_username_and_company(&admin_config.username, None)
                    .await
                {
                    Ok(None) => {
                        info!("Admin user not found. Creating...");
                        match password_encoder.hash(&admin_config.password) {
                            Ok(hash) => {
                                let new_admin = User {
                                    id: Uuid::new_v4(),
                                    username: admin_config.username.clone(),
                                    email: admin_config.email.clone(),
                                    password_hash: hash,
                                    role: admin_role,
                                    company: None,
                                    created_at: chrono::Utc::now(),
                                    updated_at: chrono::Utc::now(),
                                };
                                match user_repo.create(new_admin).await {
                                    Ok(_) => info!("Admin user seeded successfully."),
                                    Err(e) => error!("Failed to seed admin user: {}", e),
                                }
                            }
                            Err(e) => error!("Failed to hash admin password: {}", e),
                        }
                    }
                    Ok(Some(_)) => info!("Admin user already exists. Skipping."),
                    Err(e) => error!("Failed to check for existing admin user: {}", e),
                }
            }
            Ok(None) => error!("'Admin' role not found. Cannot seed admin user."),
            Err(e) => error!("Failed to fetch 'Admin' role: {}", e),
        }
    }

    // 6. Initialize Integration Clients
    info!("Initializing integration clients...");

    // 6.1 Initialize Model Prediction Client
    let model_client: Arc<dyn ModelPredictionClient> =
        match config.integrations.model_serving.provider.as_str() {
            "tensorflow" => {
                info!("Using TensorFlow Serving for model predictions");
                Arc::new(TensorFlowServingClient::new(
                    config.integrations.model_serving.url.clone(),
                    config.integrations.model_serving.model_name.clone(),
                    config.integrations.model_serving.timeout_seconds,
                    config.integrations.model_serving.image_size.unwrap_or(256),
                    config
                        .integrations
                        .model_serving
                        .concurrency_limit
                        .unwrap_or(10),
                ))
            }
            "tensorflow_grpc" => {
                info!("Using TensorFlow Serving with gRPC for model predictions");
                Arc::new(
                    TensorFlowServingGrpcClient::new(
                        config.integrations.model_serving.url.clone(),
                        config.integrations.model_serving.model_name.clone(),
                        None, // model_version opcional
                        config.integrations.model_serving.timeout_seconds,
                        config.integrations.model_serving.image_size.unwrap_or(256),
                        config
                            .integrations
                            .model_serving
                            .concurrency_limit
                            .unwrap_or(10),
                    )?,
                )
            }
            "mock" => {
                info!("Using Mock Model Client (development mode)");
                Arc::new(MockModelClient::new())
            }
            provider => {
                error!("Invalid model serving provider: {}", provider);
                anyhow::bail!(
                    "Invalid model serving provider: {}. Use 'tensorflow', 'tensorflow_grpc', or 'mock'",
                    provider
                );
            }
        };

    // 6.2 Initialize Blob Storage Client
    let storage_client: Arc<dyn BlobStorageClient> =
        match config.integrations.storage.provider.as_str() {
            "azure" => {
                info!("Using Azure Blob Storage");
                let conn_str = config
                    .integrations
                    .storage
                    .connection_string
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Azure connection string is required"))?;
                let container = config
                    .integrations
                    .storage
                    .container_name
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Azure container name is required"))?;
                Arc::new(AzureBlobClient::new(conn_str, container)?)
            }
            "local" => {
                info!("Using Local Filesystem Storage");
                let base_path = config
                    .integrations
                    .storage
                    .local_base_path
                    .clone()
                    .unwrap_or_else(|| "/tmp/spl-blobs".to_string());
                Arc::new(LocalFileSystemClient::new(base_path))
            }
            "mock" => {
                info!("Using Mock Blob Storage (development mode)");
                Arc::new(MockBlobClient::new())
            }
            provider => {
                error!("Invalid storage provider: {}", provider);
                anyhow::bail!(
                    "Invalid storage provider: {}. Use 'azure', 'local', or 'mock'",
                    provider
                );
            }
        };

    // 6.3 Health Checks
    info!("Running integration health checks...");
    model_client.health_check().await?;
    storage_client.health_check().await?;
    info!("All integrations healthy.");

    // 7. Initialize Services
    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        password_encoder.clone(),
        token_generator,
    ));

    let role_service = Arc::new(RoleService::new(role_repo.clone()));

    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        role_repo.clone(),
        company_repo.clone(),
        password_encoder,
    ));

    let company_service = Arc::new(CompanyService::new(company_repo.clone()));

    let recommendation_category_repo = Arc::new(
        repositories::recommendation::DbCategoryRepository::new(db.clone()),
    );
    let recommendation_repo = Arc::new(DbRecommendationRepository::new(
        db.clone(),
        recommendation_category_repo.clone(),
    ));

    let recommendation_category_service = Arc::new(services::recommendation::CategoryService::new(
        recommendation_category_repo,
    ));
    let recommendation_service = Arc::new(RecommendationService::new(
        recommendation_repo,
        recommendation_category_service.clone(),
    ));

    // 7.1 Initialize Diagnostics Services
    let label_repo = Arc::new(DbLabelRepository::new(db.clone()));
    let mark_type_repo = Arc::new(DbMarkTypeRepository::new(db.clone()));
    let label_service = Arc::new(LabelService::new(label_repo.clone()));
    let mark_type_service = Arc::new(MarkTypeService::new(mark_type_repo.clone()));

    // 7.2 Initialize Image Service
    let image_repo = Arc::new(DbImageRepository::new(db.clone()));
    let image_service = Arc::new(ImageService::new(image_repo.clone()));
    // 7.3 Initialize Prediction Service
    let prediction_mark_repo = Arc::new(DbPredictionMarkRepository::new(
        db.clone(),
        mark_type_repo.clone(),
    ));
    // 7.4 Initialize Plot Service
    let plot_repo = Arc::new(DbPlotRepository::new(db.clone()));
    let prediction_repo = Arc::new(DbPredictionRepository::new(
        db.clone(),
        user_repo.clone(),
        image_repo.clone(),
        label_repo.clone(),
        prediction_mark_repo.clone(),
    ));

    // 7.5 Initialize Access Control
    let access_control_service = Arc::new(services::access_control::AccessControlService::new(
        company_repo.clone(),
        user_repo.clone(),
    ));

    let prediction_service = Arc::new(services::diagnostics::PredictionService::new(
        prediction_repo.clone(),
        user_repo.clone(),
        image_repo.clone(),
        label_repo.clone(),
        prediction_mark_repo.clone(),
        mark_type_repo.clone(),
        storage_client.clone(),
        model_client.clone(),
        access_control_service.clone(),
    ));

    let plot_service = Arc::new(PlotService::new(
        plot_repo,
        prediction_repo,
        access_control_service,
    ));

    // 7. Initialize Web Router
    info!("Loading role cache...");
    let roles_list = role_repo.get_all().await?;
    let mut role_cache = std::collections::HashMap::new();
    for role in roles_list {
        role_cache.insert(role.name, role.level);
    }
    info!("Role cache loaded with {} roles.", role_cache.len());

    let app_state = Arc::new(AppState::new(
        config.clone(),
        auth_service,
        role_service,
        user_service,
        company_service,
        image_service,
        recommendation_category_service,
        recommendation_service,
        label_service,
        mark_type_service,
        prediction_service,
        plot_service,
        role_cache,
        model_client,
        storage_client,
    ));
    let app = router(app_state);

    // 8. Start Server
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid address");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
