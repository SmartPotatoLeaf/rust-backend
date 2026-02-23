use crate::entities::plot::{DetailedPlot, Plot};
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;
use uuid::Uuid;


#[async_trait]
pub trait PlotRepository: CrudRepository<Plot, Uuid> {
    /// Get all plots for a specific company
    async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<Plot>>;

    // Get all plots for a specific company, including the default plot (unassigned predictions) with id = None
    async fn get_all_by_company_id(&self, company_id: Uuid) -> Result<Vec<Plot>>;

    /// Get a plot by ID only if it belongs to the company
    async fn get_by_company_id_and_id(&self, company_id: Uuid, id: Uuid) -> Result<Option<Plot>>;

    /// Get detailed statistics for all plots of a company (paginated). 
    /// 
    /// The default plot (unassigned predictions) is included with id = None.
    async fn get_detailed(
        &self,
        company_id: Uuid,
        offset: u64,
        limit: u64,
        labels: Vec<String>,
    ) -> Result<(i64, Vec<DetailedPlot>)>;

    /// Get detailed statistics for a specific plot
    async fn get_detailed_by_id(
        &self,
        company_id: Uuid,
        plot_id: Uuid,
        labels: Vec<String>,
    ) -> Result<Option<DetailedPlot>>;

    /// Get statistics for predictions without assigned plot (default plot)
    /// This might still be user-specific or company-specific depending on requirements.
    /// Assuming Company-wide for consistency with Plot.
    async fn get_default_detailed(
        &self,
        company_id: Uuid, // Changed to company_id
        labels: Vec<String>,
    ) -> Result<Option<DetailedPlot>>;
}
