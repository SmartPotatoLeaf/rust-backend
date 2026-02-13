use chrono::{DateTime, Utc};
use crate::adapters::web::models::diagnostics::{LabelResponse, SimplifiedLabelResponse};
use crate::adapters::web::models::plot::{PlotResponse, SimplifiedPlotResponse};
use crate::adapters::web::models::user::{SimplifiedUserResponse, UserResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DashboardFiltersRequest {
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DashboardSummaryRequest {
    pub users_ids: Option<Vec<Uuid>>,
    pub min_date: Option<DateTime<Utc>>,
    pub max_date: Option<DateTime<Utc>>,
    pub plot_ids: Option<Vec<Option<Uuid>>>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardFiltersResponse {
    pub labels: Vec<LabelResponse>,
    pub plots: Vec<PlotResponse>,
    pub users: Vec<UserResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimplifiedDashboardFiltersResponse {
    pub labels: Vec<SimplifiedLabelResponse>,
    pub plots: Vec<SimplifiedPlotResponse>,
    pub users: Vec<SimplifiedUserResponse>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardLabelCountResponse {
    pub label: SimplifiedLabelResponse,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardDistributionResponse {
    pub month: String,
    pub labels: Vec<DashboardLabelCountResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardSummaryResponse {
    pub total: u64,
    pub plots: u64,
    pub mean_severity: f32,
    pub distribution: Option<Vec<DashboardDistributionResponse>>,
}
