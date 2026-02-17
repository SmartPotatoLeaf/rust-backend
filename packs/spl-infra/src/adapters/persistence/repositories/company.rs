use crate::adapters::persistence::entities::company;
use sea_orm::*;
use spl_domain::entities::company::Company;
use spl_domain::ports::repositories::company::CompanyRepository;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::Result;
use uuid::Uuid;

pub struct DbCompanyRepository {
    db: DatabaseConnection,
}

impl DbCompanyRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CrudRepository<Company, Uuid> for DbCompanyRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Company>> {
        crud::get_by_id::<company::Entity, Company, Uuid>(&self.db, id).await
    }

    async fn create(&self, entity: Company) -> Result<Company> {
        crud::create::<company::Entity, Company>(&self.db, entity).await
    }

    async fn update(&self, entity: Company) -> Result<Company> {
        crud::update::<company::Entity, Company>(&self.db, entity).await
    }

    async fn delete(&self, id: Uuid) -> Result<Company> {
        crud::delete::<company::Entity, Company, Uuid>(&self.db, id).await
    }
}

#[async_trait::async_trait]
impl CompanyRepository for DbCompanyRepository {
    async fn get_all(&self) -> Result<Vec<Company>> {
        crud::get_all::<company::Entity, Company>(&self.db).await
    }
}
