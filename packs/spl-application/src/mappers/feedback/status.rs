use crate::dtos::feedback::status::{CreateFeedbackStatusDto, UpdateFeedbackStatusDto};
use chrono::Utc;
use spl_domain::entities::feedback::FeedbackStatus;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;

impl From<CreateFeedbackStatusDto> for FeedbackStatus {
    fn from(dto: CreateFeedbackStatusDto) -> Self {
        Self {
            id: 0,
            name: dto.name,
            description: dto.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl IntoWithContext<FeedbackStatus, FeedbackStatus> for UpdateFeedbackStatusDto {
    type Error = AppError;

    fn into_with_context(self, context: FeedbackStatus) -> Result<FeedbackStatus> {
        Ok(FeedbackStatus {
            name: self.name.unwrap_or(context.name),
            description: self.description.or(context.description),
            updated_at: Utc::now(),
            ..context
        })
    }
}
