use chrono::{DateTime, Utc};
use crate::adapters::web::models::diagnostics::{LabelResponse, SimplifiedLabelResponse};
use crate::adapters::web::models::plot::{PlotResponse, SimplifiedPlotResponse};
use crate::adapters::web::models::user::{SimplifiedUserResponse, UserResponse};
use crate::adapters::web::models::diagnostics::prediction::{PredictionResponse, SimplifiedPredictionResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DashboardCountsRequest {
    /// Filter by specific user IDs
    pub users_ids: Option<Vec<Uuid>>,
    /// Filter data from this date onwards
    pub min_date: Option<DateTime<Utc>>,
    /// Filter data up to this date
    pub max_date: Option<DateTime<Utc>>,
    /// Filter by plot IDs (None for unassigned)
    pub plot_ids: Option<Vec<Option<Uuid>>>,
    /// Filter by disease label names
    pub labels: Option<Vec<String>>,
    /// Number of last predictions to include (default: 10)
    #[serde(default = "default_last_n")]
    pub last_n: u64,
}

fn default_last_n() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DashboardFiltersRequest {
    /// Filter by company ID
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DashboardSummaryRequest {
    /// Filter by specific user IDs
    pub users_ids: Option<Vec<Uuid>>,
    /// Filter data from this date onwards
    pub min_date: Option<DateTime<Utc>>,
    /// Filter data up to this date
    pub max_date: Option<DateTime<Utc>>,
    /// Filter by plot IDs (None for unassigned)
    pub plot_ids: Option<Vec<Option<Uuid>>>,
    /// Filter by disease label names
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardFiltersResponse {
    /// Available disease labels
    pub labels: Vec<LabelResponse>,
    /// Available plots
    pub plots: Vec<PlotResponse>,
    /// Available users
    pub users: Vec<UserResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimplifiedDashboardFiltersResponse {
    /// Available disease labels (simplified)
    pub labels: Vec<SimplifiedLabelResponse>,
    /// Available plots (simplified)
    pub plots: Vec<SimplifiedPlotResponse>,
    /// Available users (simplified)
    pub users: Vec<SimplifiedUserResponse>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardLabelCountResponse {
    /// Disease label
    pub label: SimplifiedLabelResponse,
    /// Number of predictions with this label
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardDistributionResponse {
    /// Month in format YYYY-MM
    pub month: String,
    /// Label counts for this month
    pub labels: Vec<DashboardLabelCountResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardSummaryResponse {
    /// Total number of predictions
    pub total: u64,
    /// Total number of plots
    pub plots: u64,
    /// Average severity across all predictions
    pub mean_severity: f32,
    /// Monthly distribution of predictions by label
    pub distribution: Option<Vec<DashboardDistributionResponse>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardCountsResponse {
    /// Total number of predictions
    pub total: u64,
    /// Total number of plots
    pub plots: u64,
    /// Average severity across all predictions
    pub mean_severity: f32,
    /// Monthly distribution of predictions by label
    pub distribution: Option<Vec<DashboardDistributionResponse>>,
    /// Last predictions matching the filters
    pub last_predictions: Vec<PredictionResponse>,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimplifiedDashboardCountsResponse {
    /// Total number of predictions
    pub total: u64,
    /// Total number of plots
    pub plots: u64,
    /// Average severity across all predictions
    pub mean_severity: f32,
    /// Monthly distribution of predictions by label
    pub distribution: Option<Vec<DashboardDistributionResponse>>,
    /// Last predictions matching the filters
    pub last_predictions: Vec<SimplifiedPredictionResponse>,
}

/// Request for dashboard summary with plot
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct DashboardSummaryPlotRequest {
    /// Filter by company ID (required for admin users)
    pub company_id: Option<Uuid>,
    /// Filter by specific user IDs
    pub users_ids: Option<Vec<Uuid>>,
    /// Filter data from this date onwards
    pub min_date: Option<DateTime<Utc>>,
    /// Filter data up to this date
    pub max_date: Option<DateTime<Utc>>,
    /// Filter by disease label names
    pub labels: Option<Vec<String>>,
}

/// Response for dashboard detailed plot
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DashboardDetailedPlotResponse {
    /// Plot ID
    pub id: Option<Uuid>,
    /// Plot name
    pub name: String,
    /// Plot description
    pub description: Option<String>,
    /// Plot creation date
    pub created_at: DateTime<Utc>,
    /// Total diagnoses for this plot
    pub total_diagnosis: i64,
    /// Last diagnosis date
    pub last_diagnosis: Option<DateTime<Utc>>,
    /// Number of matching diagnoses
    pub matching_diagnosis: i64,
    /// Dashboard summary (simplified)
    pub summary: DashboardSummaryResponse,
}