use crate::entities::diagnostics::Label;
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;

#[async_trait]
pub trait LabelRepository: CrudRepository<Label, i32> {
    async fn get_by_name(&self, name: &str) -> Result<Option<Label>>;
    async fn get_by_severity(&self, percentage: f32) -> Result<Option<Label>>;
    async fn get_all(&self) -> Result<Vec<Label>>;
}
