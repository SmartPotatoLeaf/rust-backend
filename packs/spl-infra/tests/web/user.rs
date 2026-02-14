use crate::common::build_app;
use crate::common::mocks::*;
use axum::http::{Request, StatusCode};
use axum::body::Body;
use mockall::predicate::*;
use spl_domain::entities::user::{Role, User};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_me_endpoint_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mock_role_repo = MockRoleRepository::new();
    let mock_company_repo = MockCompanyRepository::new();
    let mock_encoder = MockPasswordEncoder::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();

    let user = User {
        id: user_id,
        username: "meuser".to_string(),
        email: Some("me@example.com".to_string()),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
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

    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(1)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(1)
        .returning(move |_| Ok(Some(user.clone())));

    let app = build_app(
        mock_user_repo,
        mock_role_repo,
        mock_company_repo,
        mock_encoder,
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/users/me")
                .method("GET")
                .header("Authorization", "Bearer valid_token")
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

    assert_eq!(body_json["username"], "meuser");
    assert_eq!(body_json["email"], "me@example.com");
    assert_eq!(body_json["id"], user_id.to_string());
}

