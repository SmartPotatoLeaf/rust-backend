use crate::entities::recommendation::{self, Recommendation};
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait]
pub trait CategoryRepository: CrudRepository<recommendation::Category, i32> {
    async fn get_all(&self) -> Result<Vec<recommendation::Category>>;
}

#[async_trait]
pub trait RecommendationRepository: CrudRepository<Recommendation, Uuid> {
    async fn get_all(&self) -> Result<Vec<Recommendation>>;
    async fn get_by_severity(&self, percentage: f32) -> Result<Vec<Recommendation>>;
}
