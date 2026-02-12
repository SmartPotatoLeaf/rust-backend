use crate::dtos::diagnostics::{CreateLabelDto, UpdateLabelDto};
use chrono::Utc;
use spl_domain::entities::diagnostics::Label;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;

impl From<CreateLabelDto> for Label {
    fn from(dto: CreateLabelDto) -> Self {
        Self {
            id: 0,
            name: dto.name,
            description: dto.description,
            min: dto.min,
            max: dto.max,
            weight: dto.weight,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl IntoWithContext<Label, Label> for UpdateLabelDto {
    type Error = AppError;

    fn into_with_context(self, context: Label) -> Result<Label> {
        Ok(Label {
            name: self.name.unwrap_or(context.name),
            description: self.description.or(context.description),
            min: self.min.unwrap_or(context.min),
            max: self.max.unwrap_or(context.max),
            weight: self.weight.unwrap_or(context.weight),
            updated_at: Utc::now(),
            ..context
        })
    }
}
