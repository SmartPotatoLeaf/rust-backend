use spl_shared::config::{AppConfig, DatabaseConfig, IntegrationsConfig, ModelServingConfig, ServerConfig, StorageConfig};

pub fn create_config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".into(),
            port: 8080,
            jwt_secret: "test_secret".into(),
            jwt_expiration_hours: 24,
            cors_allowed_origins: None,
        },

        database: DatabaseConfig {
            url: "".into(),
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            max_lifetime: None,
        },
        admin: None,
        redis: None,
        integrations: IntegrationsConfig {
            model_serving: ModelServingConfig {
                provider: "tensorflow".to_string(),
                url: "".to_string(),
                model_name: "".to_string(),
                timeout_seconds: 0,
                image_size: Some(256),
                concurrency_limit: None,
            },
            storage: StorageConfig {
                provider: "azure".to_string(),
                connection_string: None,
                container_name: None,
                local_base_path: None,
            },
        },
        rate_limiting: None,
    }
}
