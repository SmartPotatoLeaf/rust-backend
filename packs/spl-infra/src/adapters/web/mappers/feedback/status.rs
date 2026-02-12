use crate::adapters::web::models::feedback::status::{
    CreateFeedbackStatusRequest, FeedbackStatusResponse, SimplifiedFeedbackStatusResponse,
    UpdateFeedbackStatusRequest,
};
use spl_application::dtos::feedback::status::{CreateFeedbackStatusDto, UpdateFeedbackStatusDto};
use spl_domain::entities::feedback::FeedbackStatus;

impl From<CreateFeedbackStatusRequest> for CreateFeedbackStatusDto {
    fn from(req: CreateFeedbackStatusRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<UpdateFeedbackStatusRequest> for UpdateFeedbackStatusDto {
    fn from(req: UpdateFeedbackStatusRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<FeedbackStatus> for FeedbackStatusResponse {
    fn from(status: FeedbackStatus) -> Self {
        Self {
            id: status.id,
            name: status.name,
            description: status.description,
            created_at: status.created_at,
            updated_at: status.updated_at,
        }
    }
}

impl From<FeedbackStatus> for SimplifiedFeedbackStatusResponse {
    fn from(status: FeedbackStatus) -> Self {
        Self {
            id: status.id,
            name: status.name,
        }
    }
}

