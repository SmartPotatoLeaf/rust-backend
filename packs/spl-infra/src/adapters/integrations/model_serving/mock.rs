use async_trait::async_trait;
use spl_domain::ports::integrations::{IntegrationClient, ModelPredictionClient, PredictionResult};
use spl_shared::error::Result;
use std::sync::{Arc, Mutex};

/// Mock model prediction client for testing
#[derive(Clone)]
pub struct MockModelClient {
    responses: Arc<Mutex<Vec<Result<PredictionResult>>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockModelClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(Vec::new())),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Add a predefined response for the next predict() call
    pub fn push_response(&self, response: Result<PredictionResult>) {
        self.responses.lock().unwrap().push(response);
    }

    /// Get the number of times predict() was called
    pub fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    /// Create a default successful prediction result
    fn default_prediction() -> PredictionResult {
        use bytes::Bytes;

        PredictionResult {
            image: Bytes::from(vec![0u8; 100]),
            leaf_mask: Bytes::from(vec![0u8; 100]),
            lesion_mask: Bytes::from(vec![0u8; 100]),
            leaf_confidence: 0.85,
            lesion_confidence: 0.75,
            severity: 45.0,
        }
    }
}

impl Default for MockModelClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationClient for MockModelClient {
    fn name(&self) -> &'static str {
        "mock_model_client"
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ModelPredictionClient for MockModelClient {
    async fn predict(&self, _image_bytes: &[u8]) -> Result<PredictionResult> {
        *self.call_count.lock().unwrap() += 1;

        let mut responses = self.responses.lock().unwrap();
        if !responses.is_empty() {
            responses.remove(0)
        } else {
            Ok(Self::default_prediction())
        }
    }

    fn get_image_size(&self) -> u32 {
        256
    }
}
