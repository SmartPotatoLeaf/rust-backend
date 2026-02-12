use crate::dtos::recommendation::{CreateRecommendationDto, UpdateRecommendationDto};
use crate::mappers::recommendation::RecommendationCreationContext;
use spl_domain::entities::recommendation::{Category, Recommendation};
use spl_domain::ports::repositories::recommendation::RecommendationRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

use super::category::CategoryService;

pub struct RecommendationService {
    repo: Arc<dyn RecommendationRepository>,
    category_service: Arc<CategoryService>,
}

impl RecommendationService {
    pub fn new(
        repo: Arc<dyn RecommendationRepository>,
        category_service: Arc<CategoryService>,
    ) -> Self {
        Self {
            repo,
            category_service,
        }
    }

    pub async fn get_all(&self) -> Result<Vec<Recommendation>> {
        self.repo.get_all().await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Recommendation>> {
        self.repo.get_by_id(id).await
    }

    pub async fn get_by_severity(&self, percentage: f32) -> Result<Vec<Recommendation>> {
        self.repo.get_by_severity(percentage).await
    }

    async fn find_category(&self, category_id: i32) -> Result<Category> {
        self.category_service
            .get_by_id(category_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "RecommendationCategory with id {} not found",
                    category_id
                ))
            })
    }

    pub async fn create(&self, dto: CreateRecommendationDto) -> Result<Recommendation> {
        // Validate recommendation type existence
        let category_id = dto.category_id;
        let recommendation_category = self.find_category(category_id).await?;

        let context = RecommendationCreationContext {
            category: recommendation_category,
        };

        let new_rec = dto.into_with_context(context)?;
        self.repo.create(new_rec).await
    }

    pub async fn update(&self, id: Uuid, dto: UpdateRecommendationDto) -> Result<Recommendation> {
        let mut current = self
            .repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Recommendation not found".to_string()))?;

        current.description = dto.description.or(current.description);
        current.min_severity = dto.min_severity.unwrap_or(current.min_severity);
        current.max_severity = dto.max_severity.unwrap_or(current.max_severity);

        // If category_id is provided, validate and update it
        if let Some(tid) = dto.category_id {
            let category = self.find_category(tid).await?;
            current.category = category;
        }

        current.updated_at = chrono::Utc::now();

        self.repo.update(current).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<Recommendation> {
        self.repo.delete(id).await
    }
}
