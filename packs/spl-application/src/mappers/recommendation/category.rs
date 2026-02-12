use crate::dtos::recommendation::category::{CreateCategoryDto, UpdateCategoryDto};
use chrono::Utc;
use spl_domain::entities::recommendation::Category;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;

impl From<CreateCategoryDto> for Category {
    fn from(dto: CreateCategoryDto) -> Self {
        Self {
            id: 0,
            name: dto.name,
            description: dto.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl IntoWithContext<Category, Category> for UpdateCategoryDto {
    type Error = AppError;

    fn into_with_context(self, context: Category) -> Result<Category> {
        Ok(Category {
            name: self.name.unwrap_or(context.name),
            description: self.description.or(context.description),
            updated_at: Utc::now(),
            ..context
        })
    }
}
