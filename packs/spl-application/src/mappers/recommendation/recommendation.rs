use crate::dtos::recommendation::CreateRecommendationDto;
use chrono::Utc;
use spl_domain::entities::recommendation::Category;
use spl_domain::entities::recommendation::Recommendation;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;

pub struct RecommendationCreationContext {
    pub category: Category,
}

impl IntoWithContext<Recommendation, RecommendationCreationContext> for CreateRecommendationDto {
    type Error = AppError;

    fn into_with_context(self, context: RecommendationCreationContext) -> Result<Recommendation> {
        Ok(Recommendation {
            id: Uuid::new_v4(),
            description: self.description,
            min_severity: self.min_severity,
            max_severity: self.max_severity,
            category: context.category,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
