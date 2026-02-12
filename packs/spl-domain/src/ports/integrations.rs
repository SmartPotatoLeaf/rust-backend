use async_trait::async_trait;
use bytes::Bytes;
use spl_shared::error::Result;

/// Base trait for all external integrations.
/// Provides common capabilities like naming and health checks.
#[async_trait]
pub trait IntegrationClient: Send + Sync {
    /// Returns the name of this integration (e.g., "tensorflow_serving", "azure_blob")
    fn name(&self) -> &'static str;

    /// Health check - returns Ok(()) if integration is reachable
    async fn health_check(&self) -> Result<()>;
}

/// Represents the result of a model prediction
#[derive(Debug, Clone)]
pub struct PredictionResult {
    /// Original resized image
    pub image: Bytes,
    /// Leaf segmentation mask
    pub leaf_mask: Bytes,
    /// Lesion segmentation mask
    pub lesion_mask: Bytes,
    /// Leaf detection confidence (0.0 to 1.0)
    pub leaf_confidence: f32,
    /// Lesion detection confidence (0.0 to 1.0)
    pub lesion_confidence: f32,
    /// Disease severity percentage (0.0 to 100.0)
    pub severity: f32,
}

/// Port for ML model prediction services
#[async_trait]
pub trait ModelPredictionClient: IntegrationClient {
    /// Predict disease presence and severity from image bytes
    async fn predict(&self, image_bytes: &[u8]) -> Result<PredictionResult>;

    /// Get the expected image size (width/height) for the model
    fn get_image_size(&self) -> u32;
}

/// Port for blob storage operations
#[async_trait]
pub trait BlobStorageClient: IntegrationClient {
    /// Upload file content to storage and return the public URL/path
    async fn upload(&self, file_content: Bytes, destination: &str) -> Result<String>;

    /// Download file content from storage
    async fn download(&self, source: &str) -> Result<Bytes>;

    /// Delete a single file from storage
    async fn delete(&self, path: &str) -> Result<()>;

    /// Delete all files under a directory prefix
    async fn delete_directory(&self, prefix: &str) -> Result<()>;
}
