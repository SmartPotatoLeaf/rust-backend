use crate::common::build_app;
use crate::common::mocks::{
    MockCompanyRepository, MockPasswordEncoder, MockRoleRepository, MockTokenGenerator,
    MockUserRepository,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mockall::predicate::*;
use spl_domain::entities::user::{Role, User};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_login_endpoint_success() {
    let mut mock_repo = MockUserRepository::new();
    let mut mock_encoder = MockPasswordEncoder::new();
    let mut mock_token = MockTokenGenerator::new();

    let user = User {
        id: Uuid::new_v4(),
        username: "webuser".to_string(),
        email: "web@example.com".to_string(),
        password_hash: "hashed".to_string(),
        role: Role {
            id: 2,
            name: "user".to_string(),
            level: 10,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Setup Mocks
    mock_repo
        .expect_get_by_username_and_company()
        .with(eq("webuser"), eq(None)) // Login with no company_id provided in payload (default)
        .times(1)
        .returning(move |_, _| Ok(Some(user.clone())));

    mock_encoder
        .expect_verify()
        .with(eq("password123"), eq("hashed"))
        .times(1)
        .returning(|_, _| Ok(true));

    mock_token
        .expect_generate()
        .times(1)
        .returning(|_, _| Ok("mocked_jwt_token".to_string()));

    let mock_role_repo = MockRoleRepository::new();
    let mock_company_repo = MockCompanyRepository::new();

    let app = build_app(
        mock_repo,
        mock_role_repo,
        mock_company_repo,
        mock_encoder,
        mock_token,
    );

    let login_payload = serde_json::json!({
        "username": "webuser",
        "password": "password123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body_json["token"], "mocked_jwt_token");
}
