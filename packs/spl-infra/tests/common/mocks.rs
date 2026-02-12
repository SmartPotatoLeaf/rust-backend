use async_trait::async_trait;
use mockall::mock;
use spl_domain::entities::diagnostics::{MarkType, Prediction, PredictionMark};
use spl_domain::ports::auth::{PasswordEncoder, TokenGenerator};
use spl_domain::ports::{
    repositories,
    repositories::{
        company::CompanyRepository,
        crud::CrudRepository,
        recommendation::RecommendationRepository,
        user::{RoleRepository, UserRepository},
    },
};
use spl_domain::{
    entities,
    entities::{
        company::Company,
        recommendation::Recommendation,
        user::{Role, User},
    },
};
use spl_shared::error::Result;
use uuid::Uuid;

mock! {
    pub UserRepository {}
    #[async_trait]
    impl CrudRepository<User, Uuid> for UserRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<User>>;
        async fn create(&self, entity: User) -> Result<User>;
        async fn update(&self, entity: User) -> Result<User>;
        async fn delete(&self, id: Uuid) -> Result<User>;
    }
    #[async_trait]
    impl UserRepository for UserRepository {
        async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<User>>;
        async fn get_by_username_and_company(&self, username: &str, company_id: Option<Uuid>) -> Result<Option<User>>;
        async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<User>> ;
    }
}

mock! {
    pub PasswordEncoder {}
    impl PasswordEncoder for PasswordEncoder {
        fn hash(&self, password: &str) -> Result<String>;
        fn verify(&self, password: &str, hash: &str) -> Result<bool>;
    }
}

mock! {
    pub TokenGenerator {}
    impl TokenGenerator for TokenGenerator {
        fn generate(&self, sub: &str, claims: serde_json::Value) -> Result<String>;
        fn validate(&self, token: &str) -> Result<serde_json::Value>;
    }
}

mock! {
    pub RoleRepository {}
    #[async_trait]
    impl CrudRepository<Role, i32> for RoleRepository {
        async fn get_by_id(&self, id: i32) -> Result<Option<Role>>;
        async fn create(&self, entity: Role) -> Result<Role>;
        async fn update(&self, entity: Role) -> Result<Role>;
        async fn delete(&self, id: i32) -> Result<Role>;
    }
    #[async_trait]
    impl RoleRepository for RoleRepository {
        async fn get_by_name(&self, name: &str) -> Result<Option<Role>>;
        async fn get_all(&self) -> Result<Vec<Role>>;
    }
}

mock! {
    pub CompanyRepository {}
    #[async_trait]
    impl CrudRepository<Company, Uuid> for CompanyRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<Company>>;
        async fn create(&self, entity: Company) -> Result<Company>;
        async fn update(&self, entity: Company) -> Result<Company>;
        async fn delete(&self, id: Uuid) -> Result<Company>;
    }
    #[async_trait]
    impl CompanyRepository for CompanyRepository {}
}

mock! {
    pub RecommendationCategoryRepository {}
    #[async_trait]
    impl CrudRepository<entities::recommendation::Category, i32> for RecommendationCategoryRepository {
        async fn get_by_id(&self, id: i32) -> Result<Option<entities::recommendation::Category>>;
        async fn create(&self, entity: entities::recommendation::Category) -> Result<entities::recommendation::Category>;
        async fn update(&self, entity: entities::recommendation::Category) -> Result<entities::recommendation::Category>;
        async fn delete(&self, id: i32) -> Result<entities::recommendation::Category>;
    }
    #[async_trait]
    impl repositories::recommendation::CategoryRepository for RecommendationCategoryRepository {
        async fn get_all(&self) -> Result<Vec<entities::recommendation::Category>>;
    }
}

mock! {
    pub RecommendationRepository {}
    #[async_trait]
    impl CrudRepository<Recommendation, Uuid> for RecommendationRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<Recommendation>>;
        async fn create(&self, entity: Recommendation) -> Result<Recommendation>;
        async fn update(&self, entity: Recommendation) -> Result<Recommendation>;
        async fn delete(&self, id: Uuid) -> Result<Recommendation>;
    }
    #[async_trait]
    impl RecommendationRepository for RecommendationRepository {
        async fn get_all(&self) -> Result<Vec<Recommendation>>;
        async fn get_by_severity(&self, percentage: f32) -> Result<Vec<Recommendation>>;
    }
}

mock! {
    pub LabelRepository {}
    #[async_trait]
    impl CrudRepository<entities::diagnostics::Label, i32> for LabelRepository {
        async fn get_by_id(&self, id: i32) -> Result<Option<entities::diagnostics::Label>>;
        async fn create(&self, entity: entities::diagnostics::Label) -> Result<entities::diagnostics::Label>;
        async fn update(&self, entity: entities::diagnostics::Label) -> Result<entities::diagnostics::Label>;
        async fn delete(&self, id: i32) -> Result<entities::diagnostics::Label>;
    }
    #[async_trait]
    impl repositories::diagnostics::LabelRepository for LabelRepository {
        async fn get_by_name(&self, name: &str) -> Result<Option<entities::diagnostics::Label>>;
        async fn get_by_severity(&self, percentage: f32) -> Result<Option<entities::diagnostics::Label>>;
        async fn get_all(&self) -> Result<Vec<entities::diagnostics::Label>>;
    }
}

mock! {
    pub MarkTypeRepository {}
    #[async_trait]
    impl CrudRepository<entities::diagnostics::MarkType, i32> for MarkTypeRepository {
        async fn get_by_id(&self, id: i32) -> Result<Option<entities::diagnostics::MarkType>>;
        async fn create(&self, entity: entities::diagnostics::MarkType) -> Result<entities::diagnostics::MarkType>;
        async fn update(&self, entity: entities::diagnostics::MarkType) -> Result<entities::diagnostics::MarkType>;
        async fn delete(&self, id: i32) -> Result<entities::diagnostics::MarkType>;
    }
    #[async_trait]
    impl repositories::diagnostics::MarkTypeRepository for MarkTypeRepository {
        async fn get_by_ids(&self, ids: Vec<i32>) -> Result<Vec<MarkType>>;
        async fn get_by_name(&self, name: &str) -> Result<Option<entities::diagnostics::MarkType>>;
        async fn get_all(&self) -> Result<Vec<entities::diagnostics::MarkType>>;
    }
}

mock! {
    pub PlotRepository {}
    #[async_trait]
    impl CrudRepository<entities::plot::Plot, Uuid> for PlotRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<entities::plot::Plot>>;
        async fn create(&self, entity: entities::plot::Plot) -> Result<entities::plot::Plot>;
        async fn update(&self, entity: entities::plot::Plot) -> Result<entities::plot::Plot>;
        async fn delete(&self, id: Uuid) -> Result<entities::plot::Plot>;
    }
    #[async_trait]
    #[async_trait]
    impl repositories::plot::PlotRepository for PlotRepository {
        async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<entities::plot::Plot>>;
        async fn get_by_company_id_and_id(&self, company_id: Uuid, id: Uuid) -> Result<Option<entities::plot::Plot>>;
        async fn get_detailed(
            &self,
            company_id: Uuid,
            offset: u64,
            limit: u64,
            labels: Vec<String>,
        ) -> Result<(i64, Vec<repositories::plot::DetailedPlot>)>;
        async fn get_detailed_by_id(
            &self,
            company_id: Uuid,
            plot_id: Uuid,
            labels: Vec<String>,
        ) -> Result<Option<repositories::plot::DetailedPlot>>;
        async fn get_default_detailed(
            &self,
            company_id: Uuid,
            labels: Vec<String>,
        ) -> Result<Option<repositories::plot::DetailedPlot>>;
    }
}

mock! {
    pub PredictionRepository {}
    #[async_trait]
    impl CrudRepository<Prediction, Uuid> for PredictionRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<Prediction>>;
        async fn create(&self, entity: Prediction) -> Result<Prediction>;
        async fn update(&self, entity: Prediction) -> Result<Prediction>;
        async fn delete(&self, id: Uuid) -> Result<Prediction>;
    }
    #[async_trait]
    impl repositories::diagnostics::PredictionRepository for PredictionRepository {
        async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Prediction>>;
        async fn assign_plot_by_ids_and_user_id(
            &self,
            prediction_ids: Vec<Uuid>,
            user_id: Uuid,
            plot_id: Option<Uuid>,
        ) -> Result<Vec<Prediction>>;
        async fn has_unassigned_predictions(&self, user_id: Uuid) -> Result<bool>;
        async fn get_all(&self) -> Result<Vec<Prediction>>;
        async fn get_by_user_id_and_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Prediction>>;
        async fn filter(
            &self,
            user_id: Vec<Uuid>,
            labels: Option<Vec<String>>,
            plot_ids: Option<Vec<Option<Uuid>>>,
            min_date: Option<chrono::DateTime<chrono::Utc>>,
            max_date: Option<chrono::DateTime<chrono::Utc>>,
            offset: u64,
            limit: u64,
        ) -> Result<(u64, Vec<Prediction>)>;
    }
}

mock! {
    pub ImageRepository {}
    #[async_trait]
    impl repositories::image::ImageRepository for ImageRepository {
        async fn create(&self, image: entities::image::Image) -> Result<entities::image::Image>;
        async fn get_by_id(&self, id: Uuid) -> Result<Option<entities::image::Image>>;
        async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<entities::image::Image>>;
        async fn update(&self, image: entities::image::Image) -> Result<entities::image::Image>;
        async fn delete(&self, id: Uuid) -> Result<()>;
    }
}

mock! {
    pub PredictionMarkRepository {}
    #[async_trait]
    impl CrudRepository<entities::diagnostics::PredictionMark, Uuid> for PredictionMarkRepository {
        async fn get_by_id(&self, id: Uuid) -> Result<Option<entities::diagnostics::PredictionMark>>;
        async fn create(&self, entity: entities::diagnostics::PredictionMark) -> Result<entities::diagnostics::PredictionMark>;
        async fn update(&self, entity: entities::diagnostics::PredictionMark) -> Result<entities::diagnostics::PredictionMark>;
        async fn delete(&self, id: Uuid) -> Result<entities::diagnostics::PredictionMark>;
    }
    #[async_trait]
    impl repositories::diagnostics::PredictionMarkRepository for PredictionMarkRepository {
        async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<PredictionMark>>;
        async fn create_many(&self, marks: Vec<entities::diagnostics::PredictionMark>) -> Result<Vec<entities::diagnostics::PredictionMark>>;
        async fn get_by_prediction_id(&self, prediction_id: Uuid) -> Result<Vec<entities::diagnostics::PredictionMark>>;
        async fn get_by_predictions_ids(&self, prediction_id: Vec<Uuid>) -> Result<Vec<entities::diagnostics::PredictionMark>>;
    }
}
