use crate::adapters::web::models::feedback::status::{
    CreateFeedbackStatusRequest, FeedbackStatusResponse, SimplifiedFeedbackStatusResponse,
    UpdateFeedbackStatusRequest,
};
use spl_application::dtos::feedback::status::{CreateFeedbackStatusDto, UpdateFeedbackStatusDto};
use spl_domain::entities::feedback::FeedbackStatus;
use spl_shared::{map_mirror, maps_to};

map_mirror!(CreateFeedbackStatusRequest, CreateFeedbackStatusDto {
    name,
    description,
});

map_mirror!(UpdateFeedbackStatusRequest, UpdateFeedbackStatusDto {
    name,
    description,
});

map_mirror!(FeedbackStatus, FeedbackStatusResponse {
    id,
    name,
    description,
    created_at,
    updated_at,
});

maps_to!(SimplifiedFeedbackStatusResponse {
    id,
    name,
} #from [ FeedbackStatus ]);

