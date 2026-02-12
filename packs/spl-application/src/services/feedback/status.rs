use crate::dtos::feedback::status::{CreateFeedbackStatusDto, UpdateFeedbackStatusDto};
use spl_domain::entities::feedback::FeedbackStatus;
use spl_domain::ports::repositories::feedback::FeedbackStatusRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;

pub struct FeedbackStatusService {
    repo: Arc<dyn FeedbackStatusRepository>,
}

impl FeedbackStatusService {
    pub fn new(repo: Arc<dyn FeedbackStatusRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<FeedbackStatus>> {
        self.repo.get_all().await
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<FeedbackStatus>> {
        self.repo.get_by_id(id).await
    }

    pub async fn create(&self, dto: CreateFeedbackStatusDto) -> Result<FeedbackStatus> {
        self.repo.create(dto.into()).await
    }

    pub async fn update(&self, id: i32, dto: UpdateFeedbackStatusDto) -> Result<FeedbackStatus> {
        let current = self
            .repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("FeedbackStatus not found".to_string()))?;

        let updated = dto.into_with_context(current)?;

        self.repo.update(updated).await
    }

    pub async fn delete(&self, id: i32) -> Result<FeedbackStatus> {
        self.repo.delete(id).await
    }
}
