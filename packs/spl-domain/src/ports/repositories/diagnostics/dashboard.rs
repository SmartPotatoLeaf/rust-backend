use async_trait::async_trait;
use chrono::{DateTime, Utc};
use spl_shared::error::Result;
use uuid::Uuid;

/// Dashboard summary aggregated data
#[derive(Debug, Clone)]
pub struct DashboardSummaryData {
    pub total: i64,
    pub plot_count: i64,
    pub labels_count: Vec<LabelCount>,
    pub monthly_distribution: Vec<MonthlyLabelCount>,
}

/// Count of predictions per label with label metadata
#[derive(Debug, Clone)]
pub struct LabelCount {
    pub label_id: i32,
    pub label_name: String,
    pub label_weight: i32,
    pub count: i64,
}

/// Count of predictions per month and label
#[derive(Debug, Clone)]
pub struct MonthlyLabelCount {
    pub month: String, // Format: "YYYY-MM"
    pub label_id: i32,
    pub label_name: String,
    pub count: i64,
}

/// Repository for dashboard analytics operations
#[async_trait]
pub trait DashboardRepository: Send + Sync {
    /// Get dashboard summary with aggregated statistics
    async fn get_summary(
        &self,
        user_id: Uuid,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Option<Vec<Option<Uuid>>>,
        labels: Option<Vec<String>>,
    ) -> Result<DashboardSummaryData>;
}
