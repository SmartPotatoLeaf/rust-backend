fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    tonic_build::configure()
        .build_server(false)
        .compile(
            &[
                "../../proto/tensorflow/core/framework/error_codes.proto",
                "../../proto/tensorflow/core/framework/status.proto",
                "../../proto/tensorflow/core/framework/tensor.proto",
                "../../proto/tensorflow/core/framework/tensor_shape.proto",
                "../../proto/tensorflow/core/framework/types.proto",
                "../../proto/tensorflow/core/framework/resource_handle.proto",
                "../../proto/tensorflow/core/framework/get_model_status.proto",
                "../../proto/tensorflow/core/framework/model_service.proto",
                "../../proto/tensorflow/serving/model.proto",
                "../../proto/tensorflow/serving/predict.proto",
                "../../proto/tensorflow/serving/get_model_metadata.proto",
                "../../proto/tensorflow/serving/prediction_service.proto",
            ],
            &["../../proto"],
        )?;
    Ok(())
}
