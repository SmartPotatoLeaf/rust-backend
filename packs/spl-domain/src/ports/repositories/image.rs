use async_trait::async_trait;
use uuid::Uuid;

use crate::entities::image::Image;
use spl_shared::error::Result;

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn create(&self, image: Image) -> Result<Image>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Image>>;
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Image>>;
    async fn update(&self, image: Image) -> Result<Image>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}
