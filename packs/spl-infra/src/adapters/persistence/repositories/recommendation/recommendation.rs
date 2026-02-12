use async_trait::async_trait;
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Select};
use spl_domain::entities::recommendation::{Category, Recommendation};
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::recommendation::{
    CategoryRepository, RecommendationRepository,
};
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

use crate::adapters::persistence::entities::recommendation::{self, category};
use crate::adapters::persistence::mappers::recommendation::recommendation::RecommendationMapperContext;

pub struct DbRecommendationRepository {
    db: DatabaseConnection,
    category_repository: Arc<dyn CategoryRepository>,
}

impl DbRecommendationRepository {
    pub fn new(db: DatabaseConnection, category_repository: Arc<dyn CategoryRepository>) -> Self {
        Self {
            db,
            category_repository,
        }
    }

    async fn find_relations(&self, category_id: i32) -> Result<Category> {
        self.category_repository
            .get_by_id(category_id)
            .await?
            .ok_or(AppError::NotFound(format!(
                "No recommendation category with id {}",
                category_id
            )))
    }

    async fn validate_relations(&self, recommendation: &Recommendation) -> Result<Category> {
        let category_id = recommendation.category.id;

        self.find_relations(category_id).await
    }

    fn map_related_model(
        &self,
        result: Option<(recommendation::Model, Option<category::Model>)>,
    ) -> Result<Option<Recommendation>> {
        match result {
            Some((model, Some(category_model))) => {
                let category: Category = category_model.into();
                let context = RecommendationMapperContext { category };
                let rec = model.into_with_context(context)?;
                Ok(Some(rec))
            }
            Some((_, None)) => Err(AppError::NotFound(
                "Recommendation found without Category".into(),
            )),
            None => Ok(None),
        }
    }

    async fn with_map<F>(&self, action: F) -> Result<Recommendation>
    where
        F: AsyncFnOnce() -> Result<(recommendation::Model, Category)>,
    {
        let (model, category) = action().await?;

        let context = RecommendationMapperContext { category };
        let rec = model.into_with_context(context)?;
        Ok(rec)
    }

    async fn find(&self, select: Select<recommendation::Entity>) -> Result<Vec<Recommendation>> {
        select
            .find_also_related(category::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from)?
            .into_iter()
            .map(|it| {
                self.map_related_model(Some(it)).and_then(|it| {
                    it.ok_or_else(|| {
                        AppError::NotFound("Recommendation found without Category".to_string())
                    })
                })
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl CrudRepository<Recommendation, Uuid> for DbRecommendationRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Recommendation>> {
        let res = self
            .find(recommendation::Entity::find_by_id(id))
            .await?
            .first()
            .cloned();

        Ok(res)
    }

    async fn create(&self, entity: Recommendation) -> Result<Recommendation> {
        self.with_map(|| async {
            let category = self.validate_relations(&entity).await?;
            let model = crud::create_model::<recommendation::Entity, Recommendation>(
                &self.db,
                entity.clone(),
            )
            .await?;
            Ok((model, category))
        })
        .await
    }

    async fn update(&self, entity: Recommendation) -> Result<Recommendation> {
        self.with_map(|| async {
            let category = self.validate_relations(&entity).await?;
            let model = crud::update_model::<recommendation::Entity, Recommendation>(
                &self.db,
                entity.clone(),
            )
            .await?;
            Ok((model, category))
        })
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<Recommendation> {
        self.with_map(|| async {
            let model =
                crud::delete_model::<recommendation::Entity, Recommendation, Uuid>(&self.db, id)
                    .await?;
            let category = self.find_relations(model.category_id).await?;
            Ok((model, category))
        })
        .await
    }
}

#[async_trait]
impl RecommendationRepository for DbRecommendationRepository {
    async fn get_all(&self) -> Result<Vec<Recommendation>> {
        self.find(recommendation::Entity::find()).await
    }

    async fn get_by_severity(&self, percentage: f32) -> Result<Vec<Recommendation>> {
        self.find(
            recommendation::Entity::find().filter(
                Condition::all()
                    .add(recommendation::Column::MinSeverity.lte(percentage))
                    .add(recommendation::Column::MaxSeverity.gte(percentage)),
            ),
        )
        .await
    }
}
