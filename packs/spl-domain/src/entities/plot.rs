use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a plot/field where predictions can be grouped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plot {
    pub id: Uuid,
    /// Company that owns this plot
    pub company_id: Uuid,
    /// Name of the plot (3-64 characters)
    pub name: String,
    /// Optional description of the plot
    pub description: Option<String>,
    /// When the plot was created
    pub created_at: DateTime<Utc>,
    /// When the plot was last updated
    pub updated_at: DateTime<Utc>,
}

/// Detailed plot statistics for aggregated views
#[derive(Debug, Clone)]
pub struct DetailedPlot {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub total_diagnosis: i64,
    pub last_diagnosis: Option<DateTime<Utc>>,
    pub matching_diagnosis: i64,
}
