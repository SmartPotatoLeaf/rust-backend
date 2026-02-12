use crate::entities::feedback::{Feedback, FeedbackStatus};
use crate::ports::repositories::crud::CrudRepository;
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait FeedbackStatusRepository: CrudRepository<FeedbackStatus, i32> {
    async fn get_all(&self) -> Result<Vec<FeedbackStatus>>;
    async fn get_by_name(&self, name: &str) -> Result<Option<FeedbackStatus>>;
}

#[async_trait::async_trait]
pub trait FeedbackRepository: CrudRepository<Feedback, Uuid> {
    async fn get_all_by_user_id(&self, user_id: Uuid) -> Result<Vec<Feedback>>;
    async fn get_by_id_and_user_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Feedback>>;
    async fn delete_by_id_and_user_id(&self, id: Uuid, user_id: Uuid) -> Result<Feedback>;
    async fn get_by_prediction_id(&self, prediction_id: Uuid) -> Result<Option<Feedback>>;
    async fn get_by_user_and_prediction_id(
        &self,
        user_id: Uuid,
        prediction_id: Uuid,
    ) -> Result<Option<Feedback>>;
}
