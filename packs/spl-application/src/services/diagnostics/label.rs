use crate::dtos::diagnostics::{CreateLabelDto, UpdateLabelDto};

use spl_domain::entities::diagnostics::Label;
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::diagnostics::LabelRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;

pub struct LabelService {
    label_repo: Arc<dyn LabelRepository>,
}

impl LabelService {
    pub fn new(label_repo: Arc<dyn LabelRepository>) -> Self {
        Self { label_repo }
    }

    pub async fn create(&self, requester: &User, dto: CreateLabelDto) -> Result<Label> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.label_repo.create(dto.into()).await
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<Label>> {
        self.label_repo.get_by_id(id).await
    }

    pub async fn get_all(&self) -> Result<Vec<Label>> {
        self.label_repo.get_all().await
    }

    pub async fn get_by_severity(&self, percentage: f32) -> Result<Option<Label>> {
        self.label_repo.get_by_severity(percentage).await
    }

    pub async fn update(&self, requester: &User, id: i32, dto: UpdateLabelDto) -> Result<Label> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }

        let current = self
            .label_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Label not found".to_string()))?;

        let updated = dto.into_with_context(current)?;
        self.label_repo.update(updated).await
    }

    pub async fn delete(&self, requester: &User, id: i32) -> Result<Label> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.label_repo.delete(id).await
    }
}
