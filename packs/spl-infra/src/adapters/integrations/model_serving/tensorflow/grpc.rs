use crate::adapters::integrations::model_serving::tensorflow::common::{
    build_prediction_result, preprocess_image_to_tensor,
};
use crate::tensorflow::serving::model_service_client::ModelServiceClient;
use crate::tensorflow::serving::model_spec::VersionChoice;
use crate::tensorflow::serving::model_version_status::State;
use crate::tensorflow::serving::prediction_service_client::PredictionServiceClient;
use crate::tensorflow::serving::{
    GetModelStatusRequest, ModelSpec, PredictRequest, PredictResponse,
};
use crate::tensorflow::tensor_shape_proto::Dim;
use crate::tensorflow::{DataType, TensorProto, TensorShapeProto};
use async_trait::async_trait;
use spl_domain::ports::integrations::{IntegrationClient, ModelPredictionClient, PredictionResult};
use spl_shared::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;

pub struct TensorFlowServingGrpcClient {
    prediction_client: PredictionServiceClient<Channel>,
    model_client: ModelServiceClient<Channel>,
    model_name: String,
    model_version: Option<i64>,
    image_size: u32,
    semaphore: Arc<Semaphore>,
}

impl TensorFlowServingGrpcClient {
    pub fn new(
        grpc_url: String,
        model_name: String,
        model_version: Option<i64>,
        timeout_seconds: u64,
        image_size: u32,
        concurrency_limit: usize,
    ) -> Result<Self> {
        let endpoint = Endpoint::from_shared(grpc_url)
            .map_err(|e| AppError::IntegrationError {
                integration: "tensorflow_serving_grpc".to_string(),
                message: format!("Invalid gRPC URL: {}", e),
            })?
            .timeout(Duration::from_secs(timeout_seconds))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .keep_alive_timeout(Duration::from_secs(10));

        // Crear el channel de forma lazy - no se conecta hasta el primer uso
        let channel = endpoint.connect_lazy();

        let prediction_client = PredictionServiceClient::new(channel.clone());
        let model_client = ModelServiceClient::new(channel);

        Ok(Self {
            prediction_client,
            model_client,
            model_name,
            model_version,
            image_size,
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
        })
    }

    fn create_spec(&self) -> ModelSpec {
        let mut model_spec = ModelSpec::default();
        model_spec.name = self.model_name.clone();
        // model_spec.signature_name = "serving_default".to_string();
        if let Some(version) = self.model_version {
            model_spec.version_choice = Some(VersionChoice::Version(version));
        }
        model_spec
    }

    fn create_request(&self, tensor_data: &[Vec<Vec<f32>>], size: &u32) -> Result<PredictRequest> {
        let model_spec = self.create_spec();

        let mut flat_data: Vec<f32> = Vec::new();
        for row in tensor_data {
            for col in row {
                for &value in col {
                    flat_data.push(value);
                }
            }
        }

        let tensor_shape = TensorShapeProto {
            dim: vec![
                Dim {
                    size: 1,
                    name: String::new(),
                },
                Dim {
                    size: *size as i64,
                    name: String::new(),
                },
                Dim {
                    size: *size as i64,
                    name: String::new(),
                },
                Dim {
                    size: 3,
                    name: String::new(),
                },
            ],
            unknown_rank: false,
        };

        let tensor_proto = TensorProto {
            dtype: DataType::DtFloat as i32,
            tensor_shape: Some(tensor_shape),
            float_val: flat_data,
            ..Default::default()
        };

        let mut inputs = HashMap::new();
        inputs.insert("input_1".to_string(), tensor_proto);

        Ok(PredictRequest {
            model_spec: Some(model_spec),
            inputs,
            output_filter: vec![],
        })
    }
}

#[async_trait]
impl IntegrationClient for TensorFlowServingGrpcClient {
    fn name(&self) -> &'static str {
        "tensorflow_serving_grpc"
    }

    async fn health_check(&self) -> Result<()> {
        let mut client = self.model_client.clone();

        // Crear ModelSpec para el health check
        let model_spec = self.create_spec();

        let request = Request::new(GetModelStatusRequest {
            model_spec: Some(model_spec),
        });

        let response =
            client
                .get_model_status(request)
                .await
                .map_err(|e| AppError::IntegrationError {
                    integration: self.name().to_string(),
                    message: format!("Health check failed: {}", e),
                })?;

        let status_response = response.into_inner();

        // Verify that we have at least one model version and that it's in AVAILABLE state
        if status_response.model_version_status.is_empty() {
            return Err(AppError::IntegrationError {
                integration: self.name().to_string(),
                message: "No model versions available".to_string(),
            });
        }

        let has_available = status_response
            .model_version_status
            .iter()
            .any(|v| v.state == State::Available as i32);

        if has_available {
            Ok(())
        } else {
            Err(AppError::IntegrationError {
                integration: self.name().to_string(),
                message: "Model is not in AVAILABLE state".to_string(),
            })
        }
    }
}

#[async_trait]
impl ModelPredictionClient for TensorFlowServingGrpcClient {
    async fn predict(&self, image_bytes: &[u8]) -> Result<PredictionResult> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to acquire semaphore: {}", e)))?;

        let size = self.get_image_size();
        let preprocessed = preprocess_image_to_tensor(image_bytes, &size)?;

        let grpc_request = self.create_request(&preprocessed.data, &size)?;

        let mut client = self.prediction_client.clone();
        let request = Request::new(grpc_request);

        let response: tonic::Response<PredictResponse> =
            client
                .predict(request)
                .await
                .map_err(|e| AppError::IntegrationError {
                    integration: self.name().to_string(),
                    message: format!("gRPC prediction failed: {}", e),
                })?;

        let predict_response = response.into_inner();
        let (output_0, output_1) = parse_grpc_response(self, predict_response)?;

        build_prediction_result(
            &output_0,
            &output_1,
            &preprocessed.resized_image_bytes,
            &size,
        )
    }

    fn get_image_size(&self) -> u32 {
        self.image_size
    }
}

fn parse_grpc_response(
    client: &dyn IntegrationClient,
    response: PredictResponse,
) -> Result<(Vec<Vec<Vec<f32>>>, Vec<Vec<Vec<f32>>>)> {
    let output_0_tensor =
        response
            .outputs
            .get("output_0")
            .ok_or_else(|| AppError::IntegrationError {
                integration: client.name().to_string(),
                message: "Missing output_0 in gRPC response".to_string(),
            })?;

    let output_1_tensor =
        response
            .outputs
            .get("output_1")
            .ok_or_else(|| AppError::IntegrationError {
                integration: client.name().to_string(),
                message: "Missing output_1 in gRPC response".to_string(),
            })?;

    let output_0 = tensor_proto_to_3d_array(client, output_0_tensor)?;
    let output_1 = tensor_proto_to_3d_array(client, output_1_tensor)?;

    Ok((output_0, output_1))
}

fn tensor_proto_to_3d_array(
    client: &dyn IntegrationClient,
    tensor: &TensorProto,
) -> Result<Vec<Vec<Vec<f32>>>> {
    let shape = tensor
        .tensor_shape
        .as_ref()
        .ok_or_else(|| AppError::IntegrationError {
            integration: client.name().to_string(),
            message: "Missing tensor shape in response".to_string(),
        })?;

    if shape.dim.len() < 3 {
        return Err(AppError::IntegrationError {
            integration: client.name().to_string(),
            message: format!("Invalid tensor shape dimensions: {}", shape.dim.len()),
        });
    }

    let height = shape.dim[1].size as usize;
    let width = shape.dim[2].size as usize;
    let channels = if shape.dim.len() > 3 {
        shape.dim[3].size as usize
    } else {
        1
    };

    let flat_data = &tensor.float_val;

    if flat_data.is_empty() {
        return Err(AppError::IntegrationError {
            integration: client.name().to_string(),
            message: "Empty tensor data in response".to_string(),
        });
    }

    let mut result = vec![vec![vec![0.0f32; channels]; width]; height];
    let mut idx = 0;

    for h in 0..height {
        for w in 0..width {
            for c in 0..channels {
                if idx < flat_data.len() {
                    result[h][w][c] = flat_data[idx];
                    idx += 1;
                }
            }
        }
    }

    Ok(result)
}
