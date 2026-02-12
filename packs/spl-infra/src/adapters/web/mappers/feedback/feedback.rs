use crate::adapters::web::models::feedback::feedback::{
    CreateFeedbackRequest, FeedbackResponse, SimplifiedFeedbackResponse, UpdateFeedbackRequest,
};
use spl_application::dtos::feedback::feedback::{CreateFeedbackDto, UpdateFeedbackDto};
use spl_domain::entities::feedback::Feedback;

impl From<CreateFeedbackRequest> for CreateFeedbackDto {
    fn from(req: CreateFeedbackRequest) -> Self {
        Self {
            comment: req.comment,
            correct_label_id: req.correct_label_id,
            prediction_id: req.prediction_id,
        }
    }
}

impl From<UpdateFeedbackRequest> for UpdateFeedbackDto {
    fn from(req: UpdateFeedbackRequest) -> Self {
        Self {
            comment: req.comment,
            correct_label_id: req.correct_label_id,
        }
    }
}

impl From<Feedback> for FeedbackResponse {
    fn from(feedback: Feedback) -> Self {
        Self {
            id: feedback.id,
            comment: feedback.comment,
            status: feedback.status.into(),
            correct_label: feedback.correct_label.map(Into::into),
            prediction_id: feedback.prediction_id,
            created_at: feedback.created_at,
            updated_at: feedback.updated_at,
        }
    }
}

impl From<Feedback> for SimplifiedFeedbackResponse {
    fn from(feedback: Feedback) -> Self {
        Self {
            id: feedback.id,
            comment: feedback.comment,
            prediction_id: feedback.prediction_id,
            correct_label: feedback.correct_label.map(Into::into),
            status: feedback.status.into(),
            updated_at: feedback.updated_at,
        }
    }
}

