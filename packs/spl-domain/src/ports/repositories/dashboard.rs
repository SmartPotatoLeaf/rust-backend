use crate::entities::dashboard::{DashboardCounts, DashboardDetailedPlot, DashboardSummary};
use chrono::{DateTime, Utc};
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait DashboardSummaryRepository: Send + Sync {
    async fn get_summary(
        &self,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<DashboardSummary>;

    async fn get_counts(
        &self,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
        last_n: u64,
    ) -> Result<DashboardCounts>;

    async fn get_summary_detailed_plot_by_id(
        &self,
        company_id: Uuid,
        plot_id: Uuid,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<Option<DashboardDetailedPlot>>;

    async fn get_default_summary_detailed_plot(
        &self,
        company_id: Uuid,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<Option<DashboardDetailedPlot>>;
}
