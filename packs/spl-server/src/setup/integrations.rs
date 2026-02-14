use anyhow::Result;
use spl_domain::ports::integrations::{BlobStorageClient, ModelPredictionClient};
use spl_infra::adapters::integrations::{
    model_serving::{
        mock::MockModelClient,
        tensorflow::{TensorFlowServingClient, TensorFlowServingGrpcClient},
    },
    storage::{azure::AzureBlobClient, local::LocalFileSystemClient, mock::MockBlobClient},
};
use spl_shared::config::IntegrationsConfig;
use std::sync::Arc;
use tracing::{error, info};

pub async fn initialize_model_client(
    config: &IntegrationsConfig,
) -> Result<Arc<dyn ModelPredictionClient>> {
    let model_config = &config.model_serving;
    let model_client: Arc<dyn ModelPredictionClient> =
        match model_config.provider.as_str() {
            "tensorflow" => {
                info!("Using TensorFlow Serving for model predictions");
                Arc::new(TensorFlowServingClient::new(
                    model_config.url.clone(),
                    model_config.model_name.clone(),
                    model_config.timeout_seconds,
                    model_config.image_size.unwrap_or(256),
                    model_config.concurrency_limit.unwrap_or(10),
                ))
            }
            "tensorflow_grpc" => {
                info!("Using TensorFlow Serving with gRPC for model predictions");
                Arc::new(TensorFlowServingGrpcClient::new(
                    model_config.url.clone(),
                    model_config.model_name.clone(),
                    None,
                    model_config.timeout_seconds,
                    model_config.image_size.unwrap_or(256),
                    model_config.concurrency_limit.unwrap_or(10),
                )?)
            }
            "mock" => {
                info!("Using Mock Model Client (development mode)");
                Arc::new(MockModelClient::new())
            }
            provider => {
                error!("Invalid model serving provider: {}", provider);
                anyhow::bail!(
                    "Invalid model serving provider: {}. Use 'tensorflow', 'tensorflow_grpc', or 'mock'",
                    provider
                );
            }
        };

    Ok(model_client)
}

pub async fn initialize_storage_client(
    config: &IntegrationsConfig,
) -> Result<Arc<dyn BlobStorageClient>> {
    let storage_client: Arc<dyn BlobStorageClient> = match config.storage.provider.as_str() {
        "azure" => {
            info!("Using Azure Blob Storage");
            let conn_str = config
                .storage
                .connection_string
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Azure connection string is required"))?;
            let container = config
                .storage
                .container_name
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Azure container name is required"))?;
            Arc::new(AzureBlobClient::new(conn_str, container)?)
        }
        "local" => {
            info!("Using Local Filesystem Storage");
            let base_path = config
                .storage
                .local_base_path
                .clone()
                .unwrap_or_else(|| "/tmp/spl-blobs".to_string());
            Arc::new(LocalFileSystemClient::new(base_path))
        }
        "mock" => {
            info!("Using Mock Blob Storage (development mode)");
            Arc::new(MockBlobClient::new())
        }
        provider => {
            error!("Invalid storage provider: {}", provider);
            anyhow::bail!(
                "Invalid storage provider: {}. Use 'azure', 'local', or 'mock'",
                provider
            );
        }
    };

    Ok(storage_client)
}

pub async fn health_checks(
    model_client: &Arc<dyn ModelPredictionClient>,
    storage_client: &Arc<dyn BlobStorageClient>,
) -> Result<()> {
    info!("Running integration health checks...");
    tokio::try_join!(model_client.health_check(), storage_client.health_check())?;
    info!("All integrations healthy.");
    Ok(())
}
