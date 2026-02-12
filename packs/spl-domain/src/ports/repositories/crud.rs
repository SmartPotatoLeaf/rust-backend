use async_trait::async_trait;
use spl_shared::error::Result;

#[async_trait]
pub trait CrudRepository<T, ID>: Send + Sync {
    async fn get_by_id(&self, id: ID) -> Result<Option<T>>;

    async fn create(&self, entity: T) -> Result<T>;

    async fn update(&self, entity: T) -> Result<T>;

    async fn delete(&self, id: ID) -> Result<T>;
}
