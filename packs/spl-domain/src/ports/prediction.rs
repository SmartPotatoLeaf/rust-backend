use async_trait::async_trait;
use spl_shared::error::Result;

#[async_trait]
pub trait DiseaseDetector: Send + Sync {
    async fn predict(&self, image_content: Vec<u8>) -> Result<PredictionResult>;
}

pub struct PredictionResult {
    pub presence: f32,
    pub absence: f32,
    pub severity: f32,
    pub leaf_mask: Vec<u8>,
    pub lesion_mask: Vec<u8>,
}
