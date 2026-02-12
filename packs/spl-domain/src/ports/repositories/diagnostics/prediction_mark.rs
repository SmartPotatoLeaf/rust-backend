use crate::entities::diagnostics::PredictionMark;
use crate::ports::repositories::crud::CrudRepository;
use async_trait::async_trait;
use spl_shared::error::Result;
use uuid::Uuid;

#[async_trait]
pub trait PredictionMarkRepository: CrudRepository<PredictionMark, Uuid> {
    
    async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<PredictionMark>>;
    async fn create_many(&self, marks: Vec<PredictionMark>) -> Result<Vec<PredictionMark>>;
    async fn get_by_prediction_id(&self, prediction_id: Uuid) -> Result<Vec<PredictionMark>>;
    async fn get_by_predictions_ids(&self, prediction_ids: Vec<Uuid>) -> Result<Vec<PredictionMark>>;
}
