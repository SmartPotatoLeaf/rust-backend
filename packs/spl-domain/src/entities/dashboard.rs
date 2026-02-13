use crate::entities::diagnostics::Label;
use crate::entities::plot::Plot;
use crate::entities::user::User;

#[derive(Debug, Clone)]
pub struct DashboardSummaryFilters {
    pub labels: Vec<Label>,
    pub plots: Vec<Plot>,
    pub users: Vec<User>
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
