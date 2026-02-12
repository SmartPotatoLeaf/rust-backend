use async_trait::async_trait;
use bytes::Bytes;
use spl_domain::ports::integrations::{BlobStorageClient, IntegrationClient};
use spl_shared::error::{AppError, Result};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Local filesystem client for blob storage (development/testing)
pub struct LocalFileSystemClient {
    base_path: PathBuf,
}

impl LocalFileSystemClient {
    /// Create a new local filesystem adapter
    pub fn new(base_path: String) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    /// Get the full path for a given destination
    fn get_full_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path.trim_start_matches('/'))
    }
}

#[async_trait]
impl IntegrationClient for LocalFileSystemClient {
    fn name(&self) -> &'static str {
        "local_filesystem"
    }

    async fn health_check(&self) -> Result<()> {
        // Ensure base directory exists and is writable
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| AppError::IntegrationError {
                integration: "local_filesystem".to_string(),
                message: format!("Base path not accessible: {}", e),
            })?;
        Ok(())
    }
}

#[async_trait]
impl BlobStorageClient for LocalFileSystemClient {
    async fn upload(&self, file_content: Bytes, destination: &str) -> Result<String> {
        let full_path = self.get_full_path(destination);

        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::IntegrationError {
                    integration: "local_filesystem".to_string(),
                    message: format!("Failed to create directory: {}", e),
                })?;
        }

        // Write file
        let mut file =
            fs::File::create(&full_path)
                .await
                .map_err(|e| AppError::IntegrationError {
                    integration: "local_filesystem".to_string(),
                    message: format!("Failed to create file: {}", e),
                })?;

        file.write_all(&file_content)
            .await
            .map_err(|e| AppError::IntegrationError {
                integration: "local_filesystem".to_string(),
                message: format!("Failed to write file: {}", e),
            })?;

        Ok(full_path.to_string_lossy().to_string())
    }

    async fn download(&self, source: &str) -> Result<Bytes> {
        let full_path = self.get_full_path(source);

        let content = fs::read(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::NotFound(format!("File not found: {}", source))
            } else {
                AppError::IntegrationError {
                    integration: "local_filesystem".to_string(),
                    message: format!("Failed to read file: {}", e),
                }
            }
        })?;

        Ok(Bytes::from(content))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.get_full_path(path);

        fs::remove_file(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::NotFound(format!("File not found: {}", path))
            } else {
                AppError::IntegrationError {
                    integration: "local_filesystem".to_string(),
                    message: format!("Failed to delete file: {}", e),
                }
            }
        })?;

        Ok(())
    }

    async fn delete_directory(&self, prefix: &str) -> Result<()> {
        let full_path = self.get_full_path(prefix);

        fs::remove_dir_all(&full_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::NotFound(format!("Directory not found: {}", prefix))
            } else {
                AppError::IntegrationError {
                    integration: "local_filesystem".to_string(),
                    message: format!("Failed to delete directory: {}", e),
                }
            }
        })?;

        Ok(())
    }
}
