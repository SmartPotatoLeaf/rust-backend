use crate::adapters::persistence::entities::diagnostics::mark_type;
use sea_orm::*;
use spl_domain::entities::diagnostics::MarkType;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::diagnostics::MarkTypeRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};

pub struct DbMarkTypeRepository {
    db: DatabaseConnection,
}

impl DbMarkTypeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CrudRepository<MarkType, i32> for DbMarkTypeRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<MarkType>> {
        crud::get_by_id::<mark_type::Entity, MarkType, i32>(&self.db, id).await
    }

    async fn create(&self, entity: MarkType) -> Result<MarkType> {
        crud::create::<mark_type::Entity, MarkType>(&self.db, entity).await
    }

    async fn update(&self, entity: MarkType) -> Result<MarkType> {
        crud::update::<mark_type::Entity, MarkType>(&self.db, entity).await
    }

    async fn delete(&self, id: i32) -> Result<MarkType> {
        crud::delete::<mark_type::Entity, MarkType, i32>(&self.db, id).await
    }
}

#[async_trait::async_trait]
impl MarkTypeRepository for DbMarkTypeRepository {
    async fn get_by_ids(&self, ids: Vec<i32>) -> Result<Vec<MarkType>> {
        let results = mark_type::Entity::find()
            .filter(mark_type::Column::Id.is_in(ids))
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(results.into_iter().map(Into::into).collect())
    }
    async fn get_by_name(&self, name: &str) -> Result<Option<MarkType>> {
        let model = mark_type::Entity::find()
            .filter(mark_type::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(model.map(Into::into))
    }

    async fn get_all(&self) -> Result<Vec<MarkType>> {
        let models = mark_type::Entity::find()
            .order_by_asc(mark_type::Column::Name)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(Into::into).collect())
    }
}
