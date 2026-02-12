use crate::dtos::plot::{
    AssignPlotDto, AssignedPlot, CreatePlotDto, DetailedPlotDto, PaginatedDetailedPlot,
    UpdatePlotDto,
};
use crate::services::access_control::AccessControlService;

use spl_domain::entities::plot::Plot;
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::diagnostics::PredictionRepository;
use spl_domain::ports::repositories::plot::{DetailedPlot, PlotRepository};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

pub struct PlotService {
    plot_repo: Arc<dyn PlotRepository>,
    prediction_repo: Arc<dyn PredictionRepository>,
    access_control: Arc<AccessControlService>,
}

impl PlotService {
    pub fn new(
        plot_repo: Arc<dyn PlotRepository>,
        prediction_repo: Arc<dyn PredictionRepository>,
        access_control: Arc<AccessControlService>,
    ) -> Self {
        Self {
            plot_repo,
            prediction_repo,
            access_control,
        }
    }

    /// Create a new plot for the user
    pub async fn create(&self, creator: &User, mut dto: CreatePlotDto) -> Result<Plot> {
        // 1. Authorization: Only Supervisor (>= 50) or higher can create
        if creator.role.level < 50 {
            return Err(AppError::Forbidden);
        }

        // 2. Resolve Company ID
        let target_company_id = self
            .access_control
            .validate_company_access(creator, dto.company_id)
            .await?;

        // 3. Update DTO
        dto.company_id = Some(target_company_id);

        self.plot_repo.create(dto.into()).await
    }

    /// Get all plots for the user's company
    pub async fn get_all_by_user(
        &self,
        user: &User,
        company_id: Option<Uuid>,
    ) -> Result<Vec<Plot>> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;
        self.plot_repo.get_by_company_id(target_company_id).await
    }

    /// Get a single plot by ID (scoped to user's company)
    pub async fn get_by_id(
        &self,
        user: &User,
        id: Uuid,
        company_id: Option<Uuid>,
    ) -> Result<Option<Plot>> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;
        self.plot_repo
            .get_by_company_id_and_id(target_company_id, id)
            .await
    }

    /// Update a plot (scoped to user's company)
    pub async fn update(
        &self,
        user: &User,
        id: Uuid,
        dto: UpdatePlotDto,
        company_id: Option<Uuid>,
    ) -> Result<Plot> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;

        let current = self
            .plot_repo
            .get_by_company_id_and_id(target_company_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Plot not found".to_string()))?;

        let updated = dto.into_with_context(current)?;
        self.plot_repo.update(updated).await
    }

    /// Delete a plot (scoped to user's company)
    pub async fn delete(&self, user: &User, id: Uuid, company_id: Option<Uuid>) -> Result<Plot> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;

        // Verify plot belongs to company before deleting
        let _ = self
            .plot_repo
            .get_by_company_id_and_id(target_company_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Plot not found".to_string()))?;

        self.plot_repo.delete(id).await
    }

    /// Assign predictions to a plot
    pub async fn assign_predictions(
        &self,
        user: &User,
        plot_id: Uuid,
        dto: AssignPlotDto,
        company_id: Option<Uuid>,
    ) -> Result<AssignedPlot> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;

        // Verify plot belongs to company
        let _ = self
            .plot_repo
            .get_by_company_id_and_id(target_company_id, plot_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Plot not found".to_string()))?;

        let predictions = self
            .prediction_repo
            .assign_plot_by_ids_and_user_id(dto.prediction_ids.clone(), user.id, Some(plot_id))
            .await?;

        if predictions.is_empty() {
            return Err(AppError::NotFound(
                "No predictions were assigned".to_string(),
            ));
        }

        Ok(AssignedPlot {
            prediction_ids: predictions.into_iter().map(|p| p.id).collect(),
        })
    }

    /// Unassign predictions from their plots (set plot_id to NULL)
    pub async fn unassign_predictions(
        &self,
        user: &User,
        dto: AssignPlotDto,
    ) -> Result<AssignedPlot> {
        let predictions = self
            .prediction_repo
            .assign_plot_by_ids_and_user_id(dto.prediction_ids.clone(), user.id, None)
            .await?;

        if predictions.is_empty() {
            return Err(AppError::NotFound(
                "No predictions were unassigned".to_string(),
            ));
        }

        Ok(AssignedPlot {
            prediction_ids: predictions.into_iter().map(|p| p.id).collect(),
        })
    }

    /// Get paginated detailed statistics for all plots
    pub async fn get_detailed(
        &self,
        user: &User,
        dto: DetailedPlotDto,
        company_id: Option<Uuid>,
    ) -> Result<PaginatedDetailedPlot> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;

        let page = dto.page.max(1);
        let limit = dto.limit.min(100).max(1);
        let offset = (page - 1) * limit;

        let labels = dto.labels.unwrap_or_default();

        let (total, items) = self
            .plot_repo
            .get_detailed(target_company_id, offset, limit, labels.clone())
            .await?;

        Ok(PaginatedDetailedPlot {
            total,
            page,
            limit,
            items,
        })
    }

    /// Get detailed statistics for a specific plot
    pub async fn get_detailed_by_id(
        &self,
        user: &User,
        id: Uuid,
        company_id: Option<Uuid>,
        labels: Vec<String>,
    ) -> Result<Option<DetailedPlot>> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;

        let detailed = self
            .plot_repo
            .get_detailed_by_id(target_company_id, id, labels)
            .await?;

        Ok(detailed.map(Into::into))
    }

    /// Get detailed statistics for unassigned predictions (default plot)
    pub async fn get_default_detailed(
        &self,
        user: &User,
        company_id: Option<Uuid>,
        labels: Vec<String>,
    ) -> Result<Option<DetailedPlot>> {
        let target_company_id = self
            .access_control
            .validate_company_access(user, company_id)
            .await?;

        let detailed = self
            .plot_repo
            .get_default_detailed(target_company_id, labels)
            .await?;

        Ok(detailed.map(Into::into))
    }
}
