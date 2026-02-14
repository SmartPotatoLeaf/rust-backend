use crate::setup::repositories::{Adapters, Repositories};
use spl_application::services;
use spl_application::services::feedback::status::FeedbackStatusService;
use spl_application::services::feedback::FeedbackService;
use spl_application::services::{
    auth::AuthService,
    company::CompanyService,
    diagnostics::{LabelService, MarkTypeService},
    image::ImageService,
    plot::PlotService,
    recommendation::RecommendationService,
    user::{role::RoleService, UserService},
};
use spl_domain::ports::integrations::{BlobStorageClient, ModelPredictionClient};
use std::sync::Arc;

pub struct Services {
    pub auth_service: Arc<AuthService>,
    pub role_service: Arc<RoleService>,
    pub user_service: Arc<UserService>,
    pub company_service: Arc<CompanyService>,
    pub image_service: Arc<ImageService>,
    pub label_service: Arc<LabelService>,
    pub mark_type_service: Arc<MarkTypeService>,
    pub prediction_service: Arc<services::diagnostics::PredictionService>,
    pub plot_service: Arc<PlotService>,
    pub recommendation_category_service: Arc<services::recommendation::CategoryService>,
    pub recommendation_service: Arc<RecommendationService>,
    pub dashboard_service: Arc<services::dashboard::DashboardService>,
    pub feedback_status_service: Arc<FeedbackStatusService>,
    pub feedback_service: Arc<FeedbackService>,
}

pub fn initialize_services(
    repos: &Repositories,
    adapters: &Adapters,
    model_client: Arc<dyn ModelPredictionClient>,
    storage_client: Arc<dyn BlobStorageClient>,
) -> Services {
    let auth_service = Arc::new(AuthService::new(
        repos.user_repo.clone(),
        adapters.password_encoder.clone(),
        adapters.token_generator.clone(),
    ));

    let role_service = Arc::new(RoleService::new(repos.role_repo.clone()));

    let user_service = Arc::new(UserService::new(
        repos.user_repo.clone(),
        repos.role_repo.clone(),
        repos.company_repo.clone(),
        adapters.password_encoder.clone(),
    ));

    let company_service = Arc::new(CompanyService::new(repos.company_repo.clone()));

    let recommendation_category_service = Arc::new(services::recommendation::CategoryService::new(
        repos.recommendation_category_repo.clone(),
    ));

    let recommendation_service = Arc::new(RecommendationService::new(
        repos.recommendation_repo.clone(),
        recommendation_category_service.clone(),
    ));

    let label_service = Arc::new(LabelService::new(repos.label_repo.clone()));
    let mark_type_service = Arc::new(MarkTypeService::new(repos.mark_type_repo.clone()));
    let image_service = Arc::new(ImageService::new(repos.image_repo.clone()));

    let access_control_service = Arc::new(services::access_control::AccessControlService::new(
        repos.company_repo.clone(),
        repos.user_repo.clone(),
    ));

    let prediction_service = Arc::new(services::diagnostics::PredictionService::new(
        repos.prediction_repo.clone(),
        repos.user_repo.clone(),
        repos.image_repo.clone(),
        repos.label_repo.clone(),
        repos.prediction_mark_repo.clone(),
        repos.mark_type_repo.clone(),
        storage_client,
        model_client,
        access_control_service.clone(),
    ));

    let plot_service = Arc::new(PlotService::new(
        repos.plot_repo.clone(),
        repos.prediction_repo.clone(),
        access_control_service,
    ));

    let dashboard_service = Arc::new(services::dashboard::DashboardService::new(
        repos.dashboard_repo.clone(),
        repos.label_repo.clone(),
        repos.plot_repo.clone(),
        repos.user_repo.clone(),
    ));

    let feedback_status_service =
        Arc::new(FeedbackStatusService::new(repos.feedback_status_repo.clone()));

    let feedback_service = Arc::new(FeedbackService::new(
        repos.feedback_repo.clone(),
        repos.feedback_status_repo.clone(),
        repos.label_repo.clone(),
    ));

    Services {
        auth_service,
        role_service,
        user_service,
        company_service,
        image_service,
        label_service,
        mark_type_service,
        prediction_service,
        plot_service,
        recommendation_category_service,
        recommendation_service,
        dashboard_service,
        feedback_status_service,
        feedback_service,
    }
}
