use axum::Router;
use spl_application::services::{
    access_control::AccessControlService,
    auth::AuthService,
    company::CompanyService,
    diagnostics::{LabelService, MarkTypeService, PredictionService},
    feedback::FeedbackService,
    plot::PlotService,
    recommendation,
    recommendation::RecommendationService,
    user::{role::RoleService, UserService},
};
use spl_domain::ports::integrations::{BlobStorageClient, ModelPredictionClient};
use spl_infra::adapters::integrations::{
    model_serving::mock::MockModelClient, storage::mock::MockBlobClient,
};
use spl_infra::adapters::web::{router, state::AppState};
use spl_shared::config::{AppConfig, IntegrationsConfig, ModelServingConfig, StorageConfig};
use std::sync::Arc;

pub mod mocks;
pub use mocks::*;
use spl_application::services::dashboard::DashboardService;
use spl_application::services::feedback::status::FeedbackStatusService;
use spl_application::services::image::ImageService;

pub fn build_app(
    user_repo: MockUserRepository,
    role_repo: MockRoleRepository,
    company_repo: MockCompanyRepository,
    encoder: MockPasswordEncoder,
    token_gen: MockTokenGenerator,
) -> Router {
    build_app_full(
        user_repo,
        role_repo,
        company_repo,
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        encoder,
        token_gen,
    )
}

pub fn build_app_full(
    mock_user_repo: MockUserRepository,
    mock_role_repo: MockRoleRepository,
    mock_company_repo: MockCompanyRepository,
    mock_rec_repo: MockRecommendationRepository,
    mock_rec_category_repo: MockRecommendationCategoryRepository,
    mock_label_repo: MockLabelRepository,
    mock_mark_type_repo: MockMarkTypeRepository,
    mock_plot_repo: MockPlotRepository,
    mock_prediction_repo: MockPredictionRepository,
    mock_prediction_mark_repo: MockPredictionMarkRepository,
    mock_image_repo: MockImageRepository,
    mock_encoder: MockPasswordEncoder,
    mock_token: MockTokenGenerator,
) -> Router {
    let user_repo = Arc::new(mock_user_repo);
    let role_repo = Arc::new(mock_role_repo);
    let company_repo = Arc::new(mock_company_repo);
    let encoder = Arc::new(mock_encoder);
    let token_gen = Arc::new(mock_token);

    let auth_service = Arc::new(AuthService::new(
        user_repo.clone(),
        encoder.clone(),
        token_gen,
    ));

    let role_service = Arc::new(RoleService::new(role_repo.clone()));

    let user_service = Arc::new(UserService::new(
        user_repo.clone(),
        role_repo,
        company_repo.clone(),
        encoder,
    ));

    let company_service = Arc::new(CompanyService::new(company_repo.clone()));

    let rec_repo = Arc::new(mock_rec_repo);
    let rec_category_repo = Arc::new(mock_rec_category_repo);

    let rec_category_service = Arc::new(recommendation::CategoryService::new(rec_category_repo));
    let rec_service = Arc::new(RecommendationService::new(
        rec_repo,
        rec_category_service.clone(),
    ));

    let label_repo = Arc::new(mock_label_repo);
    let mark_type_repo = Arc::new(mock_mark_type_repo);

    let label_service = Arc::new(LabelService::new(label_repo.clone()));
    let mark_type_service = Arc::new(MarkTypeService::new(mark_type_repo.clone()));

    let plot_repo = Arc::new(mock_plot_repo);
    let prediction_repo = Arc::new(mock_prediction_repo);
    let prediction_mark_repo = Arc::new(mock_prediction_mark_repo);

    // Initialize mock integration clients for tests
    let model_client: Arc<dyn ModelPredictionClient> = Arc::new(MockModelClient::new());
    let storage_client: Arc<dyn BlobStorageClient> = Arc::new(MockBlobClient::new());

    // Initialize Access Control
    let access_control_service = Arc::new(AccessControlService::new(
        company_repo.clone(),
        user_repo.clone(),
    ));

    let prediction_service = Arc::new(PredictionService::new(
        prediction_repo.clone(),
        user_repo.clone(),
        Arc::new(MockImageRepository::new()), // Temporary
        label_repo.clone(),
        prediction_mark_repo,
        mark_type_repo.clone(),
        storage_client.clone(),
        model_client.clone(),
        access_control_service.clone(),
    ));

    let plot_service = Arc::new(PlotService::new(
        plot_repo.clone(),
        prediction_repo.clone(),
        access_control_service,
    ));

    // Initialize Dashboard Service
    let dashboard_repo = Arc::new(MockDashboardSummaryRepository::new());
    let dashboard_service = Arc::new(DashboardService::new(
        dashboard_repo,
        label_repo.clone(),
        plot_repo.clone(),
        user_repo.clone(),
    ));

    let image_repo = Arc::new(mock_image_repo);
    let image_service = Arc::new(ImageService::new(image_repo));

    // Initialize feedback services
    let feedback_status_repo = Arc::new(MockFeedbackStatusRepository::new());
    let feedback_repo = Arc::new(MockFeedbackRepository::new());

    let feedback_status_service =
        Arc::new(FeedbackStatusService::new(feedback_status_repo.clone()));
    let feedback_service = Arc::new(FeedbackService::new(
        feedback_repo,
        feedback_status_repo,
        label_repo.clone(),
    ));

    let config = Arc::new(AppConfig {
        server: spl_shared::config::ServerConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            jwt_secret: "test_secret".into(),
            jwt_expiration_hours: 24,
            cors_allowed_origins: None,
        },

        database: spl_shared::config::DatabaseConfig {
            url: "".into(),
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            max_lifetime: None,
        },
        admin: None,
        integrations: IntegrationsConfig {
            model_serving: ModelServingConfig {
                provider: "tensorflow".to_string(),
                url: "".to_string(),
                model_name: "".to_string(),
                timeout_seconds: 0,
                image_size: Some(256),
                concurrency_limit: None,
            },
            storage: StorageConfig {
                provider: "azure".to_string(),
                connection_string: None,
                container_name: None,
                local_base_path: None,
            },
        },
    });

    let mut roles = std::collections::HashMap::new();
    roles.insert("admin".to_string(), 100);
    roles.insert("supervisor".to_string(), 50);
    roles.insert("user".to_string(), 10);

    let state = Arc::new(AppState::new(
        config,
        auth_service,
        role_service,
        user_service,
        company_service,
        image_service,
        rec_category_service,
        rec_service,
        label_service,
        mark_type_service,
        prediction_service,
        plot_service,
        dashboard_service,
        feedback_service,
        feedback_status_service,
        roles,
        model_client,
        storage_client,
    ));
    router(state)
}
