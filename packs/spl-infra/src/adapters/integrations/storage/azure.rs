use async_trait::async_trait;
use azure_core::error::Error as AzureError;
use azure_storage::ConnectionString;
use azure_storage_blobs::prelude::*;
use bytes::Bytes;
use spl_domain::ports::integrations::{BlobStorageClient, IntegrationClient};
use spl_shared::error::{AppError, Result};

/// Azure Blob Storage client implementation
pub struct AzureBlobClient {
    container_client: ContainerClient,
    account_name: String,
    container_name: String,
}

impl AzureBlobClient {
    /// Create a new Azure Blob Storage client from connection string
    pub fn new(connection_string: &str, container_name: &str) -> Result<Self> {
        let connection =
            ConnectionString::new(connection_string).map_err(|e| map_azure_error(e, "new"))?;

        let credentials = connection
            .storage_credentials()
            .map_err(|e| map_azure_error(e, "new"))?;

        let account_name = connection
            .account_name
            .ok_or_else(|| AppError::IntegrationError {
                integration: "azure_blob".to_string(),
                message: "Account name missing in connection string".to_string(),
            })?;

        // Parse connection string and create client
        let builder = ClientBuilder::new(account_name, credentials);

        let container_client = builder.container_client(container_name);

        Ok(Self {
            container_client,
            account_name: account_name.to_string(),
            container_name: container_name.to_string(),
        })
    }
}

#[async_trait]
impl IntegrationClient for AzureBlobClient {
    fn name(&self) -> &'static str {
        "azure_blob_storage"
    }

    async fn health_check(&self) -> Result<()> {
        self.container_client
            .get_properties()
            .await
            .map_err(|e| map_azure_error(e, "health_check"))?;
        Ok(())
    }
}

#[async_trait]
impl BlobStorageClient for AzureBlobClient {
    async fn upload(&self, file_content: Bytes, destination: &str) -> Result<String> {
        self.container_client
            .blob_client(destination)
            .put_block_blob(file_content)
            .await
            .map_err(|e| map_azure_error(e, "upload"))?;

        Ok(format!(
            "https://{}.blob.core.windows.net/{}/{}",
            self.account_name, self.container_name, destination
        ))
    }

    async fn download(&self, source: &str) -> Result<Bytes> {
        let blob_client = self.container_client.blob_client(source);
        let result = blob_client
            .get_content()
            .await
            .map_err(|e| map_azure_error(e, "download"))?;

        Ok(Bytes::from(result))
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.container_client
            .blob_client(path)
            .delete()
            .await
            .map_err(|e| map_azure_error(e, "delete"))?;
        Ok(())
    }

    async fn delete_directory(&self, prefix: &str) -> Result<()> {
        let mut stream = self
            .container_client
            .list_blobs()
            .prefix(prefix.to_string())
            .into_stream();

        use futures::StreamExt;
        while let Some(result) = stream.next().await {
            let response = result.map_err(|e| map_azure_error(e, "delete_directory"))?;

            for blob in response.blobs.blobs() {
                self.container_client
                    .blob_client(&blob.name)
                    .delete()
                    .await
                    .map_err(|e| map_azure_error(e, "delete_directory"))?;
            }
        }

        Ok(())
    }
}

/// Map Azure SDK errors to AppError
fn map_azure_error(error: AzureError, operation: &str) -> AppError {
    let error_kind = error.kind();

    match error_kind {
        azure_core::error::ErrorKind::Io => AppError::IntegrationUnavailable(format!(
            "Azure Blob Storage unavailable during {}: {}",
            operation, error
        )),
        azure_core::error::ErrorKind::HttpResponse { status, .. } if *status == 404 => {
            AppError::NotFound("Blob not found in Azure Storage".to_string())
        }
        _ => AppError::IntegrationError {
            integration: "azure_blob".to_string(),
            message: format!("{}: {}", operation, error),
        },
    }
}
