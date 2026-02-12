use crate::adapters::persistence::entities::recommendation::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::recommendation::{Category, Recommendation};
use spl_shared::error::AppError;
use spl_shared::traits::FromWithContext;

pub struct RecommendationMapperContext {
    pub category: Category,
}

impl FromWithContext<Model, RecommendationMapperContext> for Recommendation {
    type Error = AppError;

    fn from_with_context(
        model: Model,
        context: RecommendationMapperContext,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: model.id,
            description: model.description,
            min_severity: model.min_severity,
            max_severity: model.max_severity,
            category: context.category,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        })
    }
}

impl From<Recommendation> for ActiveModel {
    fn from(entity: Recommendation) -> Self {
        Self {
            id: Set(entity.id),
            description: Set(entity.description),
            category_id: Set(entity.category.id),
            min_severity: Set(entity.min_severity),
            max_severity: Set(entity.max_severity),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}
