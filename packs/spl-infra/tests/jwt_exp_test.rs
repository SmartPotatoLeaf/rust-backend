use crate::common::config::create_config;
use spl_domain::ports::auth::TokenGenerator;
use spl_infra::adapters::auth::jwt::JwtTokenGenerator;
use std::sync::Arc;

mod common;

#[test]
fn test_jwt_expiration() {
    let config = create_config();

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
