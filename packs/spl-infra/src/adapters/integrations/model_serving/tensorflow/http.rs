use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use spl_domain::ports::integrations::{IntegrationClient, ModelPredictionClient, PredictionResult};
use spl_shared::error::{AppError, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use super::super::super::http_client::RetryableHttpClient;
use super::common::{build_prediction_result, preprocess_image_to_tensor};

pub struct TensorFlowServingClient {
    http_client: RetryableHttpClient,
    base_url: String,
    model_name: String,
    image_size: u32,
    semaphore: Arc<Semaphore>,
}

impl TensorFlowServingClient {
    pub fn new(
        base_url: String,
        model_name: String,
        timeout_seconds: u64,
        image_size: u32,
        concurrency_limit: usize,
    ) -> Self {
        Self {
            http_client: RetryableHttpClient::new(
                3,
                Duration::from_secs(timeout_seconds),
            ),
            base_url: base_url.trim_end_matches('/').to_string(),
            model_name,
            image_size,
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
        }
    }
}

#[async_trait]
impl IntegrationClient for TensorFlowServingClient {
    fn name(&self) -> &'static str {
        "tensorflow_serving"
    }

    async fn health_check(&self) -> Result<()> {
        let url = format!("{}/v1/models/{}", self.base_url, self.model_name);
        self.http_client.get(&url).await.map(|_| ())
    }
}

#[async_trait]
impl ModelPredictionClient for TensorFlowServingClient {
    async fn predict(&self, image_bytes: &[u8]) -> Result<PredictionResult> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to acquire semaphore: {}", e)))?;

        let size = self.get_image_size();
        let preprocessed = preprocess_image_to_tensor(image_bytes, &size)?;

        let request = TFServingRequest {
            instances: vec![preprocessed.data.clone()],
        };

        let url = format!("{}/v1/models/{}:predict", self.base_url, self.model_name);

        let request_body = serde_json::to_vec(&request).map_err(|e| AppError::IntegrationError {
            integration: "tensorflow_serving".to_string(),
            message: format!("Failed to serialize request: {}", e),
        })?;

        let response = self.http_client.post(&url, &request_body).await?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::IntegrationError {
                integration: "tensorflow_serving".to_string(),
                message: format!("Failed to read response: {}", e),
            })?;

        let tf_response: TFServingResponse =
            serde_json::from_str(&response_text).map_err(|e| AppError::IntegrationError {
                integration: "tensorflow_serving".to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        if tf_response.predictions.is_empty() {
            return Err(AppError::IntegrationError {
                integration: "tensorflow_serving".to_string(),
                message: "Model returned no predictions".to_string(),
            });
        }

        let prediction = &tf_response.predictions[0];
        build_prediction_result(
            &prediction.output_0,
            &prediction.output_1,
            &preprocessed.resized_image_bytes,
            &size,
        )
    }

    fn get_image_size(&self) -> u32 {
        self.image_size
    }
}

#[derive(Serialize)]
struct TFServingRequest {
    instances: Vec<Vec<Vec<Vec<f32>>>>,
}

#[derive(Deserialize)]
struct TFServingResponse {
    predictions: Vec<TFPrediction>,
}

#[derive(Deserialize)]
struct TFPrediction {
    output_0: Vec<Vec<Vec<f32>>>,
    output_1: Vec<Vec<Vec<f32>>>,
}
