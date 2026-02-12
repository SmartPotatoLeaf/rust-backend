use crate::adapters::persistence::entities::image;
use sea_orm::*;
use spl_domain::entities::image::Image;
use spl_domain::ports::repositories::image::ImageRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use uuid::Uuid;

pub struct DbImageRepository {
    db: DatabaseConnection,
}

impl DbImageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ImageRepository for DbImageRepository {
    async fn create(&self, image: Image) -> Result<Image> {
        crud::create::<image::Entity, Image>(&self.db, image).await
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Image>> {
        crud::get_by_id::<image::Entity, Image, Uuid>(&self.db, id).await
    }

    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Image>> {
        let models = image::Entity::find()
            .filter(image::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn update(&self, image: Image) -> Result<Image> {
        crud::update::<image::Entity, Image>(&self.db, image).await
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        crud::delete::<image::Entity, Image, Uuid>(&self.db, id).await?;
        Ok(())
    }
}
