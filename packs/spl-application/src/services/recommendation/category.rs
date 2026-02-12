use crate::dtos::recommendation::category::{CreateCategoryDto, UpdateCategoryDto};

use spl_domain::entities::recommendation::Category;
use spl_domain::ports::repositories::recommendation::CategoryRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;

pub struct CategoryService {
    repo: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_all(&self) -> Result<Vec<Category>> {
        self.repo.get_all().await
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<Category>> {
        self.repo.get_by_id(id).await
    }

    pub async fn create(&self, dto: CreateCategoryDto) -> Result<Category> {
        self.repo.create(dto.into()).await
    }

    pub async fn update(&self, id: i32, dto: UpdateCategoryDto) -> Result<Category> {
        let current =
            self.repo.get_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound("RecommendationCategory not found".to_string())
            })?;

        let updated = dto.into_with_context(current)?;

        self.repo.update(updated).await
    }

    pub async fn delete(&self, id: i32) -> Result<Category> {
        self.repo.delete(id).await
    }
}
