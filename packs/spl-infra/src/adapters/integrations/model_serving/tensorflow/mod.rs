pub mod common;
pub mod http;
pub mod grpc;

pub use http::TensorFlowServingClient;
pub use grpc::TensorFlowServingGrpcClient;
