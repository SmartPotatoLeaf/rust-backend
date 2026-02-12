use crate::adapters::persistence::entities::diagnostics::label;
use sea_orm::*;
use spl_domain::entities::diagnostics::Label;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::diagnostics::LabelRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};

pub struct DbLabelRepository {
    db: DatabaseConnection,
}

impl DbLabelRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CrudRepository<Label, i32> for DbLabelRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<Label>> {
        crud::get_by_id::<label::Entity, Label, i32>(&self.db, id).await
    }

    async fn create(&self, entity: Label) -> Result<Label> {
        crud::create::<label::Entity, Label>(&self.db, entity).await
    }

    async fn update(&self, entity: Label) -> Result<Label> {
        crud::update::<label::Entity, Label>(&self.db, entity).await
    }

    async fn delete(&self, id: i32) -> Result<Label> {
        crud::delete::<label::Entity, Label, i32>(&self.db, id).await
    }
}

#[async_trait::async_trait]
impl LabelRepository for DbLabelRepository {
    async fn get_by_name(&self, name: &str) -> Result<Option<Label>> {
        let model = label::Entity::find()
            .filter(label::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(model.map(Into::into))
    }

    async fn get_by_severity(&self, percentage: f32) -> Result<Option<Label>> {
        let model = label::Entity::find()
            .filter(label::Column::Min.lte(percentage))
            .filter(label::Column::Max.gte(percentage))
            .order_by_asc(label::Column::Weight)
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(model.map(Into::into))
    }

    async fn get_all(&self) -> Result<Vec<Label>> {
        let models = label::Entity::find()
            .order_by_asc(label::Column::Weight)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(Into::into).collect())
    }
}
