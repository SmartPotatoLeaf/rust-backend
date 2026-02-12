use crate::adapters::persistence::entities::feedback::status;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use spl_domain::entities::feedback::FeedbackStatus;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::feedback::FeedbackStatusRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::Result;

pub struct DbFeedbackStatusRepository {
    db: DatabaseConnection,
}

impl DbFeedbackStatusRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CrudRepository<FeedbackStatus, i32> for DbFeedbackStatusRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<FeedbackStatus>> {
        crud::get_by_id::<status::Entity, FeedbackStatus, i32>(&self.db, id).await
    }

    async fn create(&self, entity: FeedbackStatus) -> Result<FeedbackStatus> {
        crud::create::<status::Entity, FeedbackStatus>(&self.db, entity).await
    }

    async fn update(&self, entity: FeedbackStatus) -> Result<FeedbackStatus> {
        crud::update::<status::Entity, FeedbackStatus>(&self.db, entity).await
    }

    async fn delete(&self, id: i32) -> Result<FeedbackStatus> {
        crud::delete::<status::Entity, FeedbackStatus, i32>(&self.db, id).await
    }
}

#[async_trait]
impl FeedbackStatusRepository for DbFeedbackStatusRepository {
    async fn get_all(&self) -> Result<Vec<FeedbackStatus>> {
        crud::get_all::<status::Entity, FeedbackStatus>(&self.db).await
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<FeedbackStatus>> {
        let result = status::Entity::find()
            .filter(status::Column::Name.eq(name))
            .one(&self.db)
            .await?;

        Ok(result.map(Into::into))
    }
}
