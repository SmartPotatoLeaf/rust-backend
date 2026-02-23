use crate::entities::diagnostics::Label;
use crate::entities::diagnostics::Prediction;
use crate::entities::plot::{DetailedPlot, Plot};
use crate::entities::user::User;
use chrono::{DateTime, Utc};
use spl_shared::error::AppError;
use spl_shared::traits::FromWithContext;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DashboardSummaryFilters {
    pub labels: Vec<Label>,
    pub plots: Vec<Plot>,
    pub users: Vec<User>,
}

#[derive(Debug, Clone)]
pub struct DashboardLabelCount {
    pub label: Label,
    pub count: u64,
}

#[derive(Debug, Clone)]
pub struct DashboardDistribution {
    pub month: String,
    pub labels: Vec<DashboardLabelCount>,
}

#[derive(Debug, Clone)]
pub struct DashboardSummary {
    pub total: u64,
    pub plots: u64,
    pub mean_severity: f32,
    pub distribution: Option<Vec<DashboardDistribution>>,
}

#[derive(Debug, Clone)]
pub struct DashboardCounts {
    pub total: u64,
    pub plots: u64,
    pub mean_severity: f32,
    pub distribution: Option<Vec<DashboardDistribution>>,
    pub last_predictions: Vec<Prediction>,
}

#[derive(Debug, Clone)]
pub struct DashboardDetailedPlot {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub total_diagnosis: i64,
    pub last_diagnosis: Option<DateTime<Utc>>,
    pub matching_diagnosis: i64,
    pub summary: DashboardSummary,
}

impl FromWithContext<DetailedPlot, DashboardSummary> for DashboardDetailedPlot {
    type Error = AppError;

    fn from_with_context(
        item: DetailedPlot,
        context: DashboardSummary,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            id: item.id,
            name: item.name,
            description: item.description,
            created_at: item.created_at,
            total_diagnosis: item.total_diagnosis,
            last_diagnosis: item.last_diagnosis,
            matching_diagnosis: item.matching_diagnosis,
            summary: context,
        })
    }
}
