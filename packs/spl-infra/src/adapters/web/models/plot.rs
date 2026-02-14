use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

/// Request to create a new plot
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePlotRequest {
    /// Optional Company ID (only for Admins)
    pub company_id: Option<Uuid>,
    /// Plot name (3-64 characters)
    #[validate(length(min = 3, max = 64))]
    pub name: String,
    /// Optional description
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

/// Request to update an existing plot
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdatePlotRequest {
    /// New name (3-64 characters)
    #[validate(length(min = 3, max = 64))]
    pub name: Option<String>,
    /// New description
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

/// Request to assign predictions to a plot
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AssignPredictionsRequest {
    /// List of prediction IDs to assign
    #[validate(length(min = 1))]
    pub prediction_ids: Vec<Uuid>,
}

/// Request for paginated detailed plots
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct DetailedPlotsRequest {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    #[validate(range(min = 1))]
    pub page: u64,
    /// Items per page
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u64,
    /// Filter by label names (optional)
    pub labels: Option<Vec<String>>,
}

fn default_page() -> u64 {
    1
}

fn default_limit() -> u64 {
    16
}

// ============ RESPONSE MODELS ============

/// Response for a single plot
#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct PlotResponse {
    /// Unique identifier of the plot
    pub id: Uuid,
    /// Company ID that owns the plot
    pub company_id: Uuid,
    /// Plot name
    pub name: String,
    /// Plot description
    pub description: Option<String>,
    /// Timestamp when the plot was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the plot was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, ToSchema, Clone, Deserialize)]
pub struct SimplifiedPlotResponse {
    /// Unique identifier of the plot
    pub id: Uuid,
    /// Plot name
    pub name: String,
}

/// Response for detailed plot with statistics
#[derive(Debug, Serialize, ToSchema)]
pub struct DetailedPlotResponse {
    /// Plot ID (None for default/unassigned plot)
    pub id: Option<Uuid>,
    /// Plot name
    pub name: String,
    /// Plot description
    pub description: Option<String>,
    /// Timestamp when the plot was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Total number of predictions
    pub total_diagnosis: i64,
    /// Last diagnosis datetime
    pub last_diagnosis: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of predictions matching the filter (or healthy if no filter)
    pub matching_diagnosis: i64,
}

/// Paginated response for detailed plots
#[derive(Debug, Serialize, ToSchema)]
pub struct DetailedPlotsResponse {
    /// Total number of plots
    pub total: i64,
    /// Current page number
    pub page: u64,
    /// Items per page
    pub limit: u64,
    /// List of detailed plot records
    pub items: Vec<DetailedPlotResponse>,
}

/// Response for assign/unassign operations
#[derive(Debug, Serialize, ToSchema)]
pub struct AssignedPlotResponse {
    /// List of prediction IDs that were assigned
    pub prediction_ids: Vec<Uuid>,
}
