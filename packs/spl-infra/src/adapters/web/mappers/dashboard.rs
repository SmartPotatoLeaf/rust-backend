use crate::adapters::web::models::dashboard::{
    DashboardCountsRequest, DashboardCountsResponse, DashboardDistributionResponse,
    DashboardFiltersRequest, DashboardFiltersResponse, DashboardLabelCountResponse,
    DashboardSummaryRequest, DashboardSummaryResponse, SimplifiedDashboardCountsResponse,
    SimplifiedDashboardFiltersResponse,
};
use spl_application::dtos::dashboard::{
    DashboardCountsDto, DashboardFiltersDto, DashboardSummaryDto,
};
use spl_domain::entities::dashboard::{
    DashboardCounts, DashboardDistribution, DashboardLabelCount, DashboardSummary,
    DashboardSummaryFilters,
};
use spl_shared::{map_mirror, maps_to};

map_mirror!(DashboardFiltersDto, DashboardFiltersRequest { company_id });

map_mirror!(
    DashboardSummaryDto,
    DashboardSummaryRequest {
        users_ids,
        min_date,
        max_date,
        plot_ids,
        labels,
    }
);

map_mirror!(
    DashboardCountsDto,
    DashboardCountsRequest {
        users_ids,
        min_date,
        max_date,
        plot_ids,
        labels,
        last_n,
    }
);

impl From<DashboardSummaryFilters> for DashboardFiltersResponse {
    fn from(value: DashboardSummaryFilters) -> Self {
        Self {
            labels: value.labels.into_iter().map(Into::into).collect(),
            plots: value.plots.into_iter().map(Into::into).collect(),
            users: value.users.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DashboardSummaryFilters> for SimplifiedDashboardFiltersResponse {
    fn from(value: DashboardSummaryFilters) -> Self {
        Self {
            labels: value.labels.into_iter().map(Into::into).collect(),
            plots: value.plots.into_iter().map(Into::into).collect(),
            users: value.users.into_iter().map(Into::into).collect(),
        }
    }
}

maps_to!(DashboardLabelCountResponse {
    count,
    #into [ label ]
} #from [DashboardLabelCount]);

impl From<DashboardDistribution> for DashboardDistributionResponse {
    fn from(value: DashboardDistribution) -> Self {
        Self {
            month: value.month,
            labels: value.labels.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DashboardSummary> for DashboardSummaryResponse {
    fn from(value: DashboardSummary) -> Self {
        Self {
            total: value.total,
            plots: value.plots,
            mean_severity: value.mean_severity,
            distribution: value
                .distribution
                .map(|c| c.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<DashboardCounts> for DashboardCountsResponse {
    fn from(value: DashboardCounts) -> Self {
        Self {
            total: value.total,
            plots: value.plots,
            mean_severity: value.mean_severity,
            distribution: value
                .distribution
                .map(|c| c.into_iter().map(Into::into).collect()),
            last_predictions: value.last_predictions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DashboardCounts> for SimplifiedDashboardCountsResponse {
    fn from(value: DashboardCounts) -> Self {
        Self {
            total: value.total,
            plots: value.plots,
            mean_severity: value.mean_severity,
            distribution: value
                .distribution
                .map(|c| c.into_iter().map(Into::into).collect()),
            last_predictions: value.last_predictions.into_iter().map(Into::into).collect(),
        }
    }
}
