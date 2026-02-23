use crate::dtos::dashboard::{
    DashboardCountsDto, DashboardFiltersDto, DashboardSummaryDto, DashboardSummaryPlotDto,
};
use spl_domain::entities::dashboard::{
    DashboardCounts, DashboardDetailedPlot, DashboardSummary, DashboardSummaryFilters,
};
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::dashboard::DashboardSummaryRepository;
use spl_domain::ports::repositories::diagnostics::LabelRepository;
use spl_domain::ports::repositories::plot::PlotRepository;
use spl_domain::ports::repositories::user::UserRepository;
use spl_shared::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct DashboardService {
    dashboard_repository: Arc<dyn DashboardSummaryRepository>,
    label_repository: Arc<dyn LabelRepository>,
    plot_repository: Arc<dyn PlotRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl DashboardService {
    pub fn new(
        dashboard_repository: Arc<dyn DashboardSummaryRepository>,
        label_repository: Arc<dyn LabelRepository>,
        plot_repository: Arc<dyn PlotRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            dashboard_repository,
            label_repository,
            plot_repository,
            user_repository,
        }
    }

    /// Get available filters for dashboard (labels and plots)
    pub async fn get_filters(
        &self,
        requester: User,
        dto: DashboardFiltersDto,
    ) -> Result<DashboardSummaryFilters> {
        let company_id;
        // Admin users must provides company
        if requester.role.level >= 100 {
            company_id = dto.company_id.ok_or_else(|| {
                AppError::ValidationError("Company ID is required for admin users".into())
            })?;
        } else if let Some(company) = requester.company {
            if let Some(provided_id) = dto.company_id {
                if company.id != provided_id {
                    return Err(AppError::Forbidden);
                }
            }
            company_id = company.id
        } else {
            return Err(AppError::Forbidden);
        }

        let (labels, plots, users) = tokio::try_join!(
            self.label_repository.get_all(),
            self.plot_repository.get_all_by_company_id(company_id),
            self.user_repository.get_by_company_id(company_id),
        )?;

        Ok(DashboardSummaryFilters {
            labels,
            plots,
            users,
        })
    }

    async fn get_allowed_user_ids(
        &self,
        requester: &User,
        ids: &Option<Vec<Uuid>>,
    ) -> Result<Vec<Uuid>> {
        let company = requester
            .company
            .clone()
            .ok_or_else(|| AppError::Forbidden)?;

        let allowed_ids = self
            .user_repository
            .get_by_company_id(company.id)
            .await?
            .into_iter()
            .map(|u| (u.id, u.id))
            .collect::<HashMap<_, _>>();

        let ids = ids.clone().unwrap_or_default();

        if ids.len() > allowed_ids.len() {
            return Err(AppError::Forbidden);
        }

        if !ids.iter().all(|id| allowed_ids.contains_key(id)) {
            return Err(AppError::Forbidden);
        }

        if ids.is_empty() {
            return Ok(allowed_ids.into_iter().map(|e| e.0).collect());
        }

        Ok(ids)
    }

    async fn get_allowed_plot_ids(
        &self,
        requester: &User,
        ids: &Option<Vec<Option<Uuid>>>,
    ) -> Result<Vec<Option<Uuid>>> {
        // For non-admin users, we also need to check if they have access to the requested plot IDs (if any)
        let company = requester
            .company
            .clone()
            .ok_or_else(|| AppError::Forbidden)?;

        let allowed_ids = self
            .plot_repository
            .get_all_by_company_id(company.id)
            .await?
            .into_iter()
            .map(|p| (p.id, p.id))
            .collect::<HashMap<_, _>>();

        if let Some(ids) = ids.clone() {
            if ids.len() > allowed_ids.len() {
                return Err(AppError::Forbidden);
            }

            if !ids
                .iter()
                .all(|id| allowed_ids.contains_key(&id.unwrap_or_default()))
            {
                return Err(AppError::Forbidden);
            }
        }

        let ids = ids.clone().unwrap_or_default();
        // If no plot IDs were provided, we should use all allowed plot IDs for the user
        if ids.is_empty() {
            let mut result: Vec<Option<Uuid>> =
                allowed_ids.into_iter().map(|e| Some(e.0)).collect();
            result.push(None); // Add None to include predictions without plot

            return Ok(result);
        }

        Ok(ids)
    }

    async fn validate_ids(
        &self,
        requester: &User,
        user_ids: &Option<Vec<Uuid>>,
        plot_ids: &Option<Vec<Option<Uuid>>>,
    ) -> Result<(Vec<Uuid>, Vec<Option<Uuid>>)> {
        let mut plots_ids: Vec<Option<Uuid>> = vec![];
        let mut users_ids: Vec<Uuid> = vec![];

        if requester.role.level < 100 {
            // For non-admin users, we need to check if they have access to the requested user IDs (if any)
            (users_ids, plots_ids) = tokio::try_join!(
                self.get_allowed_user_ids(&requester, user_ids),
                self.get_allowed_plot_ids(&requester, plot_ids)
            )?;
        }

        Ok((users_ids, plots_ids))
    }

    /// Resolve company_id from requester and optional dto company_id (admin only)
    async fn resolve_company_id(
        &self,
        requester: &User,
        dto: &DashboardSummaryPlotDto,
    ) -> Result<Uuid> {
        if requester.role.level >= 100 {
            dto.company_id.ok_or_else(|| {
                AppError::ValidationError("Company ID is required for admin users".into())
            })
        } else {
            requester
                .company
                .clone()
                .ok_or_else(|| AppError::Forbidden)
                .map(|c| c.id)
        }
    }

    /// Get dashboard summary with statistics using existing filter method
    pub async fn get_summary(
        &self,
        requester: User,
        dto: DashboardSummaryDto,
    ) -> Result<DashboardSummary> {
        let (users_ids, plots_ids) = self
            .validate_ids(&requester, &dto.users_ids, &dto.plot_ids)
            .await?;
        self.dashboard_repository
            .get_summary(users_ids, dto.min_date, dto.max_date, plots_ids, dto.labels)
            .await
    }

    /// Get dashboard counts (summary + last predictions) using existing filter method
    pub async fn get_counts(
        &self,
        requester: User,
        dto: DashboardCountsDto,
    ) -> Result<DashboardCounts> {
        let (users_ids, plots_ids) = self
            .validate_ids(&requester, &dto.users_ids, &dto.plot_ids)
            .await?;

        self.dashboard_repository
            .get_counts(
                users_ids,
                dto.min_date,
                dto.max_date,
                plots_ids,
                dto.labels,
                dto.last_n,
            )
            .await
    }

    /// Get dashboard summary with detailed plot by specific plot ID (plot_id comes from path)
    pub async fn get_summary_detailed_plot_by_id(
        &self,
        requester: User,
        plot_id: Uuid,
        dto: DashboardSummaryPlotDto,
    ) -> Result<Option<DashboardDetailedPlot>> {
        let company_id = self.resolve_company_id(&requester, &dto).await?;
        let (users_ids, _) = self
            .validate_ids(&requester, &dto.users_ids, &Some(vec![Some(plot_id)]))
            .await?;

        self.dashboard_repository
            .get_summary_detailed_plot_by_id(
                company_id,
                plot_id,
                users_ids,
                dto.min_date,
                dto.max_date,
                dto.labels,
            )
            .await
    }

    /// Get dashboard summary with default plot for the company
    pub async fn get_summary_detailed_plot_default(
        &self,
        requester: User,
        dto: DashboardSummaryPlotDto,
    ) -> Result<Option<DashboardDetailedPlot>> {
        let company_id = self.resolve_company_id(&requester, &dto).await?;
        let (users_ids, plots_ids) = self
            .validate_ids(&requester, &dto.users_ids, &None)
            .await?;

        self.dashboard_repository
            .get_default_summary_detailed_plot(
                company_id,
                users_ids,
                dto.min_date,
                dto.max_date,
                plots_ids,
                dto.labels,
            )
            .await
    }
}
