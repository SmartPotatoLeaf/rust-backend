use crate::dtos::plot::{CreatePlotDto, UpdatePlotDto};
use chrono::Utc;
use spl_domain::entities::plot::Plot;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;

impl From<CreatePlotDto> for Plot {
    fn from(dto: CreatePlotDto) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id: dto.company_id.unwrap_or_default(), // Service layer must ensure this is set or handle it. But DTO has Option.
            name: dto.name,
            description: dto.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl IntoWithContext<Plot, Plot> for UpdatePlotDto {
    type Error = AppError;

    fn into_with_context(self, context: Plot) -> Result<Plot> {
        Ok(Plot {
            name: self.name.unwrap_or(context.name),
            description: self.description.or(context.description),
            updated_at: Utc::now(),
            ..context
        })
    }
}
