use sea_orm::DatabaseConnection;
use spl_domain::ports::repositories::{
    user::{RoleRepository, UserRepository},
    company::CompanyRepository,
    image::ImageRepository,
    plot::PlotRepository,
    recommendation::{CategoryRepository, RecommendationRepository},
    feedback::{FeedbackRepository, FeedbackStatusRepository},
    dashboard::DashboardSummaryRepository,
    diagnostics::{
        LabelRepository,
        MarkTypeRepository,
        PredictionRepository,
        PredictionMarkRepository,
    },
};
use spl_domain::ports::auth::{PasswordEncoder, TokenGenerator};
use spl_infra::adapters::{
    auth::{jwt::JwtTokenGenerator, password::Argon2PasswordEncoder},
    persistence::repositories::{
        company::DbCompanyRepository,
        diagnostics::{
            DbLabelRepository, DbMarkTypeRepository, DbPredictionMarkRepository,
            DbPredictionRepository,
        },
        feedback::DbFeedbackRepository,
        image::DbImageRepository,
        plot::DbPlotRepository,
        recommendation::DbRecommendationRepository,
        user::{role::DbRoleRepository, DbUserRepository},
    },
};
use spl_infra::adapters::persistence::repositories::{
    dashboard::DbDashboardSummaryRepository,
    DbFeedbackStatusRepository,
    recommendation::DbCategoryRepository,
};
use spl_shared::config::AppConfig;
use std::sync::Arc;

pub struct Repositories {
    pub role_repo: Arc<dyn RoleRepository>,
    pub company_repo: Arc<dyn CompanyRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub image_repo: Arc<dyn ImageRepository>,
    pub label_repo: Arc<dyn LabelRepository>,
    pub mark_type_repo: Arc<dyn MarkTypeRepository>,
    pub prediction_mark_repo: Arc<dyn PredictionMarkRepository>,
    pub prediction_repo: Arc<dyn PredictionRepository>,
    pub plot_repo: Arc<dyn PlotRepository>,
    pub recommendation_category_repo: Arc<dyn CategoryRepository>,
    pub recommendation_repo: Arc<dyn RecommendationRepository>,
    pub feedback_status_repo: Arc<dyn FeedbackStatusRepository>,
    pub feedback_repo: Arc<dyn FeedbackRepository>,
    pub dashboard_repo: Arc<dyn DashboardSummaryRepository>,
}

pub struct Adapters {
    pub password_encoder: Arc<dyn PasswordEncoder>,
    pub token_generator: Arc<dyn TokenGenerator>,
}

pub fn initialize_repositories(db: DatabaseConnection) -> Repositories {
    let role_repo: Arc<dyn RoleRepository> = Arc::new(DbRoleRepository::new(db.clone()));
    let company_repo: Arc<dyn CompanyRepository> = Arc::new(DbCompanyRepository::new(db.clone()));
    let user_repo: Arc<dyn UserRepository> = Arc::new(DbUserRepository::new(
        db.clone(),
        role_repo.clone(),
        company_repo.clone(),
    ));

    let image_repo: Arc<dyn ImageRepository> = Arc::new(DbImageRepository::new(db.clone()));
    let label_repo: Arc<dyn LabelRepository> = Arc::new(DbLabelRepository::new(db.clone()));
    let mark_type_repo: Arc<dyn MarkTypeRepository> = Arc::new(DbMarkTypeRepository::new(db.clone()));
    let prediction_mark_repo: Arc<dyn PredictionMarkRepository> = Arc::new(DbPredictionMarkRepository::new(
        db.clone(),
        mark_type_repo.clone(),
    ));

    let feedback_status_repo: Arc<dyn FeedbackStatusRepository> = Arc::new(DbFeedbackStatusRepository::new(db.clone()));
    let feedback_repo: Arc<dyn FeedbackRepository> = Arc::new(DbFeedbackRepository::new(
        db.clone(),
        feedback_status_repo.clone(),
        label_repo.clone(),
    ));

    let prediction_repo: Arc<dyn PredictionRepository> = Arc::new(DbPredictionRepository::new(
        db.clone(),
        user_repo.clone(),
        image_repo.clone(),
        label_repo.clone(),
        prediction_mark_repo.clone(),
        feedback_repo.clone(),
    ));

    let plot_repo: Arc<dyn PlotRepository> = Arc::new(DbPlotRepository::new(db.clone()));

    let recommendation_category_repo: Arc<dyn CategoryRepository> = Arc::new(DbCategoryRepository::new(db.clone()));
    let recommendation_repo: Arc<dyn RecommendationRepository> = Arc::new(DbRecommendationRepository::new(
        db.clone(),
        recommendation_category_repo.clone(),
    ));

    let dashboard_repo: Arc<dyn DashboardSummaryRepository> = Arc::new(DbDashboardSummaryRepository::new(db.clone()));

    Repositories {
        role_repo,
        company_repo,
        user_repo,
        image_repo,
        label_repo,
        mark_type_repo,
        prediction_mark_repo,
        prediction_repo,
        plot_repo,
        recommendation_category_repo,
        recommendation_repo,
        feedback_status_repo,
        feedback_repo,
        dashboard_repo,
    }
}

pub fn initialize_adapters(config: Arc<AppConfig>) -> Adapters {
    let password_encoder: Arc<dyn PasswordEncoder> = Arc::new(Argon2PasswordEncoder::new());
    let token_generator: Arc<dyn TokenGenerator> = Arc::new(JwtTokenGenerator::new(config));

    Adapters {
        password_encoder,
        token_generator,
    }
}
