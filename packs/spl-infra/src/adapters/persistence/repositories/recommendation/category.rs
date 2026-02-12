use crate::adapters::persistence::entities::recommendation::category;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use spl_domain::entities::recommendation::Category;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::recommendation::CategoryRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::Result;

pub struct DbCategoryRepository {
    db: DatabaseConnection,
}

impl DbCategoryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CrudRepository<Category, i32> for DbCategoryRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<Category>> {
        crud::get_by_id::<category::Entity, Category, i32>(&self.db, id).await
    }

    async fn create(&self, entity: Category) -> Result<Category> {
        crud::create::<category::Entity, Category>(&self.db, entity).await
    }

    async fn update(&self, entity: Category) -> Result<Category> {
        crud::update::<category::Entity, Category>(&self.db, entity).await
    }

    async fn delete(&self, id: i32) -> Result<Category> {
        crud::delete::<category::Entity, Category, i32>(&self.db, id).await
    }
}

#[async_trait]
impl CategoryRepository for DbCategoryRepository {
    async fn get_all(&self) -> Result<Vec<Category>> {
        crud::get_all::<category::Entity, Category>(&self.db).await
    }
}
