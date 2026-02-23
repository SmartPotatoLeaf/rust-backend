use crate::entities::diagnostics::prediction::PredictionDetailed;
use crate::entities::diagnostics::Prediction;
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait]
pub trait PredictionRepository: CrudRepository<Prediction, Uuid> {
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Prediction>>;
    async fn get_by_user_id_and_id(&self, user_id: Uuid, id: Uuid) -> Result<Option<Prediction>>;
    async fn get_all(&self) -> Result<Vec<Prediction>>;

    /// Assign multiple predictions to a plot (or unassign if plot_id is None)
    async fn assign_plot_by_ids_and_user_id(
        &self,
        prediction_ids: Vec<Uuid>,
        user_id: Uuid,
        plot_id: Option<Uuid>,
    ) -> Result<Vec<Prediction>>;

    /// Check if user has any predictions without an assigned plot
    async fn has_unassigned_predictions(&self, user_id: Uuid) -> Result<bool>;

    /// Filter predictions
    #[allow(clippy::too_many_arguments)]
    async fn filter(
        &self,
        user_ids: Vec<Uuid>,
        labels: Option<Vec<String>>,
        plot_ids: Option<Vec<Option<Uuid>>>,
        min_date: Option<chrono::DateTime<chrono::Utc>>,
        max_date: Option<chrono::DateTime<chrono::Utc>>,
        offset: u64,
        limit: u64,
    ) -> Result<(u64, Vec<Prediction>)>;

    // Get predictions detailed including recomendations
    async fn get_detailed_by_user_id_and_id(
        &self,
        user_id: Uuid,
        prediction_id: Uuid,
    ) -> Result<Option<PredictionDetailed>>;
}
