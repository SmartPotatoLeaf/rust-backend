use crate::common::config::create_config;
use spl_domain::ports::auth::TokenGenerator;
use spl_infra::adapters::auth::jwt::JwtTokenGenerator;
use std::sync::Arc;

mod common;

#[test]
fn test_jwt_expiration() {
    let config = create_config();
    let expected_hours = config.server.jwt_expiration_hours;

    let generator = JwtTokenGenerator::new(Arc::new(config));
    let claims = serde_json::json!({
        "role": "User"
    });

    // Capture time before token generation
    let now = chrono::Utc::now().timestamp() as u64;
    let token = generator.generate("test_user", claims).unwrap();
    let decoded = generator.validate(&token).unwrap();

    let exp = decoded["exp"].as_u64().unwrap();
    let expected_exp = now + (expected_hours * 3600);

    // Allow for a small time difference (within 10 seconds)
    assert!(
        exp >= expected_exp - 10 && exp <= expected_exp + 10,
        "JWT expiration mismatch: exp={}, expected={} (±10s), configured hours={}",
        exp,
        expected_exp,
        expected_hours
    );
}
