use spl_application::services::{
    auth::AuthService,
    company::CompanyService,
    diagnostics::{LabelService, MarkTypeService, PredictionService},
    image::ImageService,
    plot::PlotService,
    recommendation,
    recommendation::RecommendationService,
    user::{RoleService, UserService},
};

use spl_domain::ports::integrations::{BlobStorageClient, ModelPredictionClient};
use spl_shared::config::AppConfig;
use std::sync::Arc;

use std::collections::HashMap;

pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth_service: Arc<AuthService>,
    pub role_service: Arc<RoleService>,
    pub user_service: Arc<UserService>,
    pub company_service: Arc<CompanyService>,
    pub image_service: Arc<ImageService>,
    pub recommendation_category_service: Arc<recommendation::CategoryService>,
    pub recommendation_service: Arc<RecommendationService>,
    pub label_service: Arc<LabelService>,
    pub mark_type_service: Arc<MarkTypeService>,
    pub prediction_service: Arc<PredictionService>,
    pub plot_service: Arc<PlotService>,
    pub roles: HashMap<String, i16>,
    // Integration clients
    pub model_client: Arc<dyn ModelPredictionClient>,
    pub storage_client: Arc<dyn BlobStorageClient>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Arc<AppConfig>,
        auth_service: Arc<AuthService>,
        role_service: Arc<RoleService>,
        user_service: Arc<UserService>,
        company_service: Arc<CompanyService>,
        image_service: Arc<ImageService>,
        recommendation_category_service: Arc<recommendation::CategoryService>,
        recommendation_service: Arc<RecommendationService>,
        label_service: Arc<LabelService>,
        mark_type_service: Arc<MarkTypeService>,
        prediction_service: Arc<PredictionService>,
        plot_service: Arc<PlotService>,
        roles: HashMap<String, i16>,
        model_client: Arc<dyn ModelPredictionClient>,
        storage_client: Arc<dyn BlobStorageClient>,
    ) -> Self {
        Self {
            config,
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
            roles,
            model_client,
            storage_client,
        }
    }
}
