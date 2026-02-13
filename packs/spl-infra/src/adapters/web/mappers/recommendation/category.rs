use crate::adapters::web::models::recommendation::{
    CreateRecommendationCategoryRequest, RecommendationCategoryResponse,
    SimplifiedRecommendationCategoryResponse, UpdateRecommendationCategoryRequest,
};

use spl_application::dtos::recommendation::category::{CreateCategoryDto, UpdateCategoryDto};

use spl_domain::entities::recommendation::Category;
use spl_shared::{map_mirror, maps_to};

map_mirror!(CreateRecommendationCategoryRequest, CreateCategoryDto {
    name,
    description,
});

map_mirror!(UpdateRecommendationCategoryRequest, UpdateCategoryDto {
    name,
    description,
});

map_mirror!(Category, RecommendationCategoryResponse {
    id,
    name,
    description,
    created_at,
    updated_at,
});

maps_to!(SimplifiedRecommendationCategoryResponse {
    id,
    name,
} #from [ Category ]);
