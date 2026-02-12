use crate::entities::diagnostics::MarkType;
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;

#[async_trait]
pub trait MarkTypeRepository: CrudRepository<MarkType, i32> {
    async fn get_by_ids(&self, ids: Vec<i32>) -> Result<Vec<MarkType>>;
    async fn get_by_name(&self, name: &str) -> Result<Option<MarkType>>;
    async fn get_all(&self) -> Result<Vec<MarkType>>;
}
