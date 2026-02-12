use crate::dtos::diagnostics::{CreateMarkTypeDto, UpdateMarkTypeDto};

use spl_domain::entities::diagnostics::MarkType;
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::diagnostics::MarkTypeRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;

pub struct MarkTypeService {
    mark_type_repo: Arc<dyn MarkTypeRepository>,
}

impl MarkTypeService {
    pub fn new(mark_type_repo: Arc<dyn MarkTypeRepository>) -> Self {
        Self { mark_type_repo }
    }

    pub async fn create(&self, requester: &User, dto: CreateMarkTypeDto) -> Result<MarkType> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.mark_type_repo.create(dto.into()).await
    }

    pub async fn get_by_id(&self, requester: &User, id: i32) -> Result<Option<MarkType>> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.mark_type_repo.get_by_id(id).await
    }

    pub async fn get_all(&self, requester: &User) -> Result<Vec<MarkType>> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.mark_type_repo.get_all().await
    }

    pub async fn update(
        &self,
        requester: &User,
        id: i32,
        dto: UpdateMarkTypeDto,
    ) -> Result<MarkType> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }

        let current = self
            .mark_type_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("MarkType not found".to_string()))?;

        let updated = dto.into_with_context(current)?;
        self.mark_type_repo.update(updated).await
    }

    pub async fn delete(&self, requester: &User, id: i32) -> Result<MarkType> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.mark_type_repo.delete(id).await
    }
}
