use crate::adapters::web::models::plot::{
    AssignPredictionsRequest, CreatePlotRequest, DetailedPlotResponse, DetailedPlotsRequest,
    DetailedPlotsResponse, PlotResponse, SimplifiedPlotResponse, UpdatePlotRequest,
};
use spl_application::dtos::plot::{
    AssignPlotDto, CreatePlotDto, DetailedPlotDto, PaginatedDetailedPlot, UpdatePlotDto,
};
use spl_domain::entities::plot::{Plot, DetailedPlot};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;
use spl_shared::{map_mirror, maps_to};

/// Context for creating a plot with user context
pub struct CreatePlotContext {
    pub company_id: Option<Uuid>,
}

impl IntoWithContext<CreatePlotDto, CreatePlotContext> for CreatePlotRequest {
    type Error = AppError;

    fn into_with_context(self, context: CreatePlotContext) -> Result<CreatePlotDto> {
        Ok(CreatePlotDto {
            company_id: self.company_id.or(context.company_id),
            name: self.name,
            description: self.description,
        })
    }
}

map_mirror!(UpdatePlotRequest, UpdatePlotDto { name, description });

map_mirror!(AssignPredictionsRequest, AssignPlotDto { prediction_ids });

map_mirror!(DetailedPlotsRequest, DetailedPlotDto { page, limit, labels });

map_mirror!(Plot, PlotResponse {
    id,
    company_id,
    name,
    description,
    created_at,
    updated_at
});

maps_to!(SimplifiedPlotResponse { id, name } #from [ Plot ]);

map_mirror!(DetailedPlot, DetailedPlotResponse {
    id,
    name,
    description,
    created_at,
    total_diagnosis,
    last_diagnosis,
    matching_diagnosis
});

impl From<PaginatedDetailedPlot> for DetailedPlotsResponse {
    fn from(paginated: PaginatedDetailedPlot) -> Self {
        Self {
            total: paginated.total,
            page: paginated.page,
            limit: paginated.limit,
            items: paginated.items.into_iter().map(Into::into).collect(),
        }
    }
}
