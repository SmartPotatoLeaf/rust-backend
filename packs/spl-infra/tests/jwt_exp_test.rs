use spl_domain::ports::auth::TokenGenerator;
use spl_infra::adapters::auth::jwt::JwtTokenGenerator;
use spl_shared::config::{
    AdminConfig, AppConfig, DatabaseConfig, IntegrationsConfig, ModelServingConfig,
    ServerConfig, StorageConfig,
};
use std::sync::Arc;

#[test]
fn test_jwt_expiration() {
    let config = AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            jwt_secret: "secret".to_string(),
            jwt_expiration_hours: 1,
            cors_allowed_origins: None,
        },
        database: DatabaseConfig {
            url: "".to_string(),
            max_connections: None,
            min_connections: None,
            connect_timeout: None,
            idle_timeout: None,
            max_lifetime: None,
        },
        admin: Some(AdminConfig {
            username: "".to_string(),
            password: "".to_string(),
            email: "".to_string(),
        }),
        integrations: IntegrationsConfig {
            model_serving: ModelServingConfig {
                provider: "mock".to_string(),
                url: "".to_string(),
                model_name: "".to_string(),
                timeout_seconds: 1,
                image_size: Some(256),
                concurrency_limit: None,
            },
            storage: StorageConfig {
                provider: "mock".to_string(),
                connection_string: None,
                container_name: None,
                local_base_path: None,
            },
        },
    };

    let generator = JwtTokenGenerator::new(Arc::new(config));
    let claims = serde_json::json!({
        "role": "User"
    });

    let token = generator.generate("test_user", claims).unwrap();
    let decoded = generator.validate(&token).unwrap();

    let exp = decoded["exp"].as_u64().unwrap();
    let now = chrono::Utc::now().timestamp() as u64;
    let expected_exp = now + 3600;

    // Allow for a small time difference
    assert!(exp >= expected_exp - 10 && exp <= expected_exp + 10);
}
