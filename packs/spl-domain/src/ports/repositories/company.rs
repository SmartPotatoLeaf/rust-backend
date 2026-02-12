use crate::entities::company::Company;
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait CompanyRepository: CrudRepository<Company, Uuid> {}
