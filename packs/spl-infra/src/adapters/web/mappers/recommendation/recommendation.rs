use crate::adapters::web::models::recommendation::{
    CreateRecommendationRequest, RecommendationResponse, SimplifiedRecommendationResponse,
    UpdateRecommendationRequest,
};

use spl_application::dtos::recommendation::{CreateRecommendationDto, UpdateRecommendationDto};
use spl_domain::entities::recommendation::Recommendation;
use spl_shared::{map_mirror, maps_to};

map_mirror!(
    CreateRecommendationRequest,
    CreateRecommendationDto {
        description,
        category_id,
        min_severity,
        max_severity,
    }
);

map_mirror!(
    UpdateRecommendationRequest,
    UpdateRecommendationDto {
        description,
        category_id,
        min_severity,
        max_severity,
    }
);

map_mirror!(Recommendation, RecommendationResponse {
    id,
    description,
    min_severity,
    max_severity,
    created_at,
    updated_at,
    #into [ category ]
});

maps_to!(SimplifiedRecommendationResponse {
    id, description,
    #into [ category ]
} #from [ Recommendation ]);
