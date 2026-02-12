use crate::adapters::web::models::recommendation::{
    CreateRecommendationCategoryRequest, RecommendationCategoryResponse,
    SimplifiedRecommendationCategoryResponse, UpdateRecommendationCategoryRequest,
};

use spl_application::dtos::recommendation::category::{CreateCategoryDto, UpdateCategoryDto};

use spl_domain::entities::recommendation::Category;

impl From<CreateRecommendationCategoryRequest> for CreateCategoryDto {
    fn from(req: CreateRecommendationCategoryRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<UpdateRecommendationCategoryRequest> for UpdateCategoryDto {
    fn from(req: UpdateRecommendationCategoryRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<Category> for RecommendationCategoryResponse {
    fn from(entity: Category) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            description: entity.description,
            created_at: entity.created_at.to_rfc3339(),
            updated_at: entity.updated_at.to_rfc3339(),
        }
    }
}

impl From<Category> for SimplifiedRecommendationCategoryResponse {
    fn from(entity: Category) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
        }
    }
}
