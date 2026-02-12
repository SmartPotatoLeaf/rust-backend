use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use spl_domain::ports::integrations::{IntegrationClient, ModelPredictionClient, PredictionResult};
use spl_shared::error::{AppError, Result};

use super::super::http_client::RetryableHttpClient;

use std::sync::Arc;
use tokio::sync::Semaphore;

/// TensorFlow Serving client for ML model predictions
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
                std::time::Duration::from_secs(timeout_seconds),
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
        // Acquire permit to limit concurrency
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to acquire semaphore: {}", e)))?;

        let size = self.get_image_size();

        // 1. Preprocess image (decode, resize, normalize)
        let preprocessed_tensor = preprocess_image_to_tensor(image_bytes, &size)?;

        // 2. Create TF Serving request payload
        let request = TFServingRequest {
            instances: vec![preprocessed_tensor.data],
        };

        let url = format!("{}/v1/models/{}:predict", self.base_url, self.model_name);

        // 3. Send HTTP POST request
        let request_body =
            serde_json::to_vec(&request).map_err(|e| AppError::IntegrationError {
                integration: "tensorflow_serving".to_string(),
                message: format!("Failed to serialize request: {}", e),
            })?;

        let response = self.http_client.post(&url, &request_body).await?;

        // 4. Parse response
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

        // 5. Convert TF response to domain PredictionResult
        convert_tf_response_to_prediction(
            tf_response,
            &preprocessed_tensor.resized_image_bytes,
            &size,
        )
    }

    fn get_image_size(&self) -> u32 {
        self.image_size
    }
}

// ===========================
// Data Structures
// ===========================

#[derive(Serialize)]
struct TFServingRequest {
    instances: Vec<Vec<Vec<Vec<f32>>>>, // [batch, height, width, channels]
}

#[derive(Deserialize)]
struct TFServingResponse {
    predictions: Vec<TFPrediction>,
}

#[derive(Deserialize)]
struct TFPrediction {
    output_0: Vec<Vec<Vec<f32>>>, // Leaf mask [height, width, channels]
    output_1: Vec<Vec<Vec<f32>>>, // Lesion mask [height, width, channels]
}

struct PreprocessedImage {
    data: Vec<Vec<Vec<f32>>>,   // Normalized tensor data
    resized_image_bytes: Bytes, // Original resized image for response
}

// ===========================
// Image Preprocessing
// ===========================

fn preprocess_image_to_tensor(image_bytes: &[u8], size: &u32) -> Result<PreprocessedImage> {
    // Decode image
    let img = image::load_from_memory(image_bytes)
        .map_err(|e| AppError::ValidationError(format!("Invalid or corrupted image: {}", e)))?;

    // Resize to 256x256 (model expected size)
    let resized = img.resize_exact(
        size.clone(),
        size.clone(),
        image::imageops::FilterType::Lanczos3,
    );

    // Convert to RGB bytes
    let rgb_image = resized.to_rgb8();
    let resized_bytes = Bytes::from(rgb_image.as_raw().clone());

    // Normalize to [0, 1] and convert to tensor format [H, W, C]
    let mut tensor = vec![vec![vec![0.0f32; 3]; size.clone() as usize]; size.clone() as usize];
    for y in 0..size.clone() {
        for x in 0..size.clone() {
            let pixel = rgb_image.get_pixel(x, y);
            tensor[y as usize][x as usize][0] = pixel[0] as f32 / 255.0;
            tensor[y as usize][x as usize][1] = pixel[1] as f32 / 255.0;
            tensor[y as usize][x as usize][2] = pixel[2] as f32 / 255.0;
        }
    }

    Ok(PreprocessedImage {
        data: tensor,
        resized_image_bytes: resized_bytes,
    })
}

// ===========================
// Response Conversion
// ===========================

fn convert_tf_response_to_prediction(
    tf_response: TFServingResponse,
    resized_image_bytes: &Bytes,
    size: &u32,
) -> Result<PredictionResult> {
    if tf_response.predictions.is_empty() {
        return Err(AppError::IntegrationError {
            integration: "tensorflow_serving".to_string(),
            message: "Model returned no predictions".to_string(),
        });
    }

    let prediction = &tf_response.predictions[0];

    // Convert probability maps to binary masks and extract metadata
    let leaf_data = extract_mask_data(&prediction.output_0)?;
    let lesion_data = extract_mask_data(&prediction.output_1)?;

    // Calculate severity as percentage of overlap
    let severity = calculate_severity(&leaf_data, &lesion_data);

    // Encode images to JPEG
    // 1. Original Resized Image (RGB8 covers size x size * 3 bytes)
    let encoded_image = encode_to_jpeg(size, size, resized_image_bytes, image::ColorType::Rgb8)?;

    // 2. Leaf Mask (L8 covers size x size * 1 byte)
    let encoded_leaf_mask =
        encode_to_jpeg(size, size, &leaf_data.binary_mask, image::ColorType::L8)?;

    // 3. Lesion Mask (L8 covers size x size * 1 byte)
    let encoded_lesion_mask =
        encode_to_jpeg(size, size, &lesion_data.binary_mask, image::ColorType::L8)?;

    Ok(PredictionResult {
        image: encoded_image,
        leaf_mask: encoded_leaf_mask,
        lesion_mask: encoded_lesion_mask,
        leaf_confidence: leaf_data.confidence,
        lesion_confidence: lesion_data.confidence,
        severity,
    })
}

struct MaskData {
    binary_mask: Vec<u8>,
    confidence: f32,
}

fn extract_mask_data(output: &[Vec<Vec<f32>>]) -> Result<MaskData> {
    let mut binary_mask = Vec::new();
    let mut prob_sum = 0.0;
    let mut above_threshold_count = 0;

    let threshold = 0.5;

    for row in output {
        for col in row {
            // Use first channel (assuming single-channel output or take max)
            let prob = col.iter().copied().fold(f32::NEG_INFINITY, f32::max);

            if prob > threshold {
                binary_mask.push(255); // White
                prob_sum += prob;
                above_threshold_count += 1;
            } else {
                binary_mask.push(0); // Black
            }
        }
    }

    let confidence = if above_threshold_count > 0 {
        prob_sum / above_threshold_count as f32
    } else {
        0.0
    };

    Ok(MaskData {
        binary_mask,
        confidence,
    })
}

fn calculate_severity(leaf: &MaskData, lesion: &MaskData) -> f32 {
    let mut overlap_count = 0;
    let mut leaf_count = 0;

    for i in 0..leaf.binary_mask.len().min(lesion.binary_mask.len()) {
        if leaf.binary_mask[i] == 255 {
            leaf_count += 1;
            if lesion.binary_mask[i] == 255 {
                overlap_count += 1;
            }
        }
    }

    if leaf_count > 0 {
        (overlap_count as f32 / leaf_count as f32) * 100.0
    } else {
        0.0
    }
}

fn encode_to_jpeg(
    width: &u32,
    height: &u32,
    data: &[u8],
    color_type: image::ColorType,
) -> Result<Bytes> {
    let mut buffer = std::io::Cursor::new(Vec::new());

    image::write_buffer_with_format(
        &mut buffer,
        data,
        width.clone(),
        height.clone(),
        color_type,
        image::ImageFormat::Jpeg,
    )
    .map_err(|e| AppError::IntegrationError {
        integration: "tensorflow_serving".to_string(),
        message: format!("Failed to encode JPEG: {}", e),
    })?;

    Ok(Bytes::from(buffer.into_inner()))
}
