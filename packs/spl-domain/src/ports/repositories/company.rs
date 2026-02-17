use crate::entities::company::Company;
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait]
pub trait CompanyRepository: CrudRepository<Company, Uuid> {
    async fn get_all(&self) -> Result<Vec<Company>>;
}
