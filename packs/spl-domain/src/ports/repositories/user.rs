use crate::entities::user::{Role, User};
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: CrudRepository<User, Uuid> {
    async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<User>>;
    async fn get_by_username_and_company(
        &self,
        username: &str,
        company_id: Option<Uuid>,
    ) -> Result<Option<User>>;

    async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<User>>;
}

#[async_trait]
pub trait RoleRepository: CrudRepository<Role, i32> {
    async fn get_by_name(&self, name: &str) -> Result<Option<Role>>;
    async fn get_all(&self) -> Result<Vec<Role>>;
}
