use async_trait::async_trait;
use bytes::Bytes;
use spl_domain::ports::integrations::{BlobStorageClient, IntegrationClient};
use spl_shared::error::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock blob storage client for testing
#[derive(Clone)]
pub struct MockBlobClient {
    storage: Arc<Mutex<HashMap<String, Bytes>>>,
}

impl MockBlobClient {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the number of stored blobs (for testing)
    pub fn count(&self) -> usize {
        self.storage.lock().unwrap().len()
    }

    /// Check if a blob exists (for testing)
    pub fn exists(&self, path: &str) -> bool {
        self.storage.lock().unwrap().contains_key(path)
    }
}

impl Default for MockBlobClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationClient for MockBlobClient {
    fn name(&self) -> &'static str {
        "mock_blob_storage"
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl BlobStorageClient for MockBlobClient {
    async fn upload(&self, file_content: Bytes, destination: &str) -> Result<String> {
        self.storage
            .lock()
            .unwrap()
            .insert(destination.to_string(), file_content);
        Ok(format!("mock://{}", destination))
    }

    async fn download(&self, source: &str) -> Result<Bytes> {
        self.storage
            .lock()
            .unwrap()
            .get(source)
            .cloned()
            .ok_or_else(|| {
                spl_shared::error::AppError::NotFound(format!("Blob not found: {}", source))
            })
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.storage.lock().unwrap().remove(path).ok_or_else(|| {
            spl_shared::error::AppError::NotFound(format!("Blob not found: {}", path))
        })?;
        Ok(())
    }

    async fn delete_directory(&self, prefix: &str) -> Result<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.retain(|k, _| !k.starts_with(prefix));
        Ok(())
    }
}
