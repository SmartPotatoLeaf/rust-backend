use crate::dtos::diagnostics::{CreateMarkTypeDto, UpdateMarkTypeDto};
use chrono::Utc;
use spl_domain::entities::diagnostics::MarkType;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;

impl From<CreateMarkTypeDto> for MarkType {
    fn from(dto: CreateMarkTypeDto) -> Self {
        Self {
            id: 0,
            name: dto.name,
            description: dto.description,
            created_at: Utc::now(),
        }
    }
}

impl IntoWithContext<MarkType, MarkType> for UpdateMarkTypeDto {
    type Error = AppError;

    fn into_with_context(self, context: MarkType) -> Result<MarkType> {
        Ok(MarkType {
            name: self.name.unwrap_or(context.name),
            description: self.description.or(context.description),
            ..context
        })
    }
}
