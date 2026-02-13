use chrono::{DateTime, Utc};
use uuid::Uuid;

/// DTO for requesting dashboard filters
#[derive(Debug, Clone)]
pub struct DashboardFiltersDto {
    pub company_id: Option<Uuid>,
}

/// DTO for requesting dashboard summary with filters
#[derive(Debug, Clone)]
pub struct DashboardSummaryDto {
    pub users_ids: Option<Vec<Uuid>>,
    pub min_date: Option<DateTime<Utc>>,
    pub max_date: Option<DateTime<Utc>>,
    pub plot_ids: Option<Vec<Option<Uuid>>>,
    pub labels: Option<Vec<String>>,
}
