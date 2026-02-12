use crate::common::build_app;
use crate::common::mocks::{
    MockCompanyRepository, MockPasswordEncoder, MockRoleRepository, MockTokenGenerator,
    MockUserRepository,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let mock_repo = MockUserRepository::new();
    let mock_encoder = MockPasswordEncoder::new();
    let mock_token = MockTokenGenerator::new();
    let mock_role_repo = MockRoleRepository::new();
    let mock_company_repo = MockCompanyRepository::new();

    let app = build_app(
        mock_repo,
        mock_role_repo,
        mock_company_repo,
        mock_encoder,
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body_json["status"], "ok");
    assert_eq!(body_json["message"], "Server is clean and running");
}
