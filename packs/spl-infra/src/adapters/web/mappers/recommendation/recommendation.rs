use crate::adapters::web::models::recommendation::{
    CreateRecommendationRequest, RecommendationResponse, SimplifiedRecommendationResponse,
    UpdateRecommendationRequest,
};

use spl_application::dtos::recommendation::{CreateRecommendationDto, UpdateRecommendationDto};
use spl_domain::entities::recommendation::Recommendation;

impl From<CreateRecommendationRequest> for CreateRecommendationDto {
    fn from(req: CreateRecommendationRequest) -> Self {
        Self {
            description: req.description,
            category_id: req.category_id,
            min_severity: req.min_severity,
            max_severity: req.max_severity,
        }
    }
}

impl From<UpdateRecommendationRequest> for UpdateRecommendationDto {
    fn from(req: UpdateRecommendationRequest) -> Self {
        Self {
            description: req.description,
            category_id: req.category_id,
            min_severity: req.min_severity,
            max_severity: req.max_severity,
        }
    }
}

impl From<Recommendation> for RecommendationResponse {
    fn from(entity: Recommendation) -> Self {
        Self {
            id: entity.id,
            description: entity.description,
            category: entity.category.into(),
            min_severity: entity.min_severity,
            max_severity: entity.max_severity,
            created_at: entity.created_at.to_rfc3339(),
            updated_at: entity.updated_at.to_rfc3339(),
        }
    }
}

impl From<Recommendation> for SimplifiedRecommendationResponse {
    fn from(entity: Recommendation) -> Self {
        Self {
            id: entity.id,
            description: entity.description,
            category: entity.category.into(),
        }
    }
}
