use crate::entities::dashboard::DashboardSummary;
use chrono::{DateTime, Utc};
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait DashboardSummaryRepository : Send + Sync {
    async fn get_summary(
        &self,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<DashboardSummary>;
}
