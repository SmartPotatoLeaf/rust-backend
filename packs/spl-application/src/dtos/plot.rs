use serde::{Deserialize, Serialize};
use spl_domain::entities::plot::DetailedPlot;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlotDto {
    pub company_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlotDto {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignPlotDto {
    pub prediction_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedPlotDto {
    pub page: u64,
    pub limit: u64,
    pub labels: Option<Vec<String>>,
}

/// Paginated response for detailed plot listings
pub struct PaginatedDetailedPlot {
    pub total: i64,
    pub page: u64,
    pub limit: u64,
    pub items: Vec<DetailedPlot>,
}

/// Response for assign/unassign operations
pub struct AssignedPlot {
    pub prediction_ids: Vec<Uuid>,
}
