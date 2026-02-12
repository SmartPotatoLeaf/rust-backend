pub mod adapters;


pub mod tensorflow {
    tonic::include_proto!("tensorflow");
    pub mod serving {
        tonic::include_proto!("tensorflow.serving");
    }
    pub mod error {
        tonic::include_proto!("tensorflow.error");
    }
}
