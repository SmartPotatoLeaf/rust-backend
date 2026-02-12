use crate::adapters::web::models::plot::{
    AssignPredictionsRequest, CreatePlotRequest, DetailedPlotResponse, DetailedPlotsRequest,
    DetailedPlotsResponse, PlotResponse, SimplifiedPlotResponse, UpdatePlotRequest,
};
use spl_application::dtos::plot::{
    AssignPlotDto, CreatePlotDto, DetailedPlotDto, PaginatedDetailedPlot, UpdatePlotDto,
};
use spl_domain::entities::plot::Plot;
use spl_domain::ports::repositories::plot::DetailedPlot;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;

/// Context for creating a plot with user context
pub struct CreatePlotContext {
    pub company_id: Option<Uuid>,
}

impl IntoWithContext<CreatePlotDto, CreatePlotContext> for CreatePlotRequest {
    type Error = AppError;

    fn into_with_context(self, context: CreatePlotContext) -> Result<CreatePlotDto> {
        Ok(CreatePlotDto {
            company_id: if self.company_id.is_some() {
                self.company_id
            } else {
                context.company_id
            },
            name: self.name,
            description: self.description,
        })
    }
}

impl From<UpdatePlotRequest> for UpdatePlotDto {
    fn from(req: UpdatePlotRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<AssignPredictionsRequest> for AssignPlotDto {
    fn from(req: AssignPredictionsRequest) -> Self {
        Self {
            prediction_ids: req.prediction_ids,
        }
    }
}

impl From<DetailedPlotsRequest> for DetailedPlotDto {
    fn from(req: DetailedPlotsRequest) -> Self {
        Self {
            page: req.page,
            limit: req.limit,
            labels: req.labels,
        }
    }
}

impl From<Plot> for PlotResponse {
    fn from(plot: Plot) -> Self {
        Self {
            id: plot.id,
            company_id: plot.company_id,
            name: plot.name,
            description: plot.description,
            created_at: plot.created_at,
            updated_at: plot.updated_at,
        }
    }
}

impl From<Plot> for SimplifiedPlotResponse {
    fn from(plot: Plot) -> Self {
        Self {
            id: plot.id,
            name: plot.name,
        }
    }
}

impl From<DetailedPlot> for DetailedPlotResponse {
    fn from(detailed: DetailedPlot) -> Self {
        Self {
            id: detailed.id,
            name: detailed.name,
            description: detailed.description,
            created_at: detailed.created_at,
            total_diagnosis: detailed.total_diagnosis,
            last_diagnosis: detailed.last_diagnosis,
            matching_diagnosis: detailed.matching_diagnosis,
        }
    }
}

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
