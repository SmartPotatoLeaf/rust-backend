use crate::common::build_app;
use crate::common::mocks::{
    MockPasswordEncoder, MockRoleRepository, MockTokenGenerator, MockUserRepository,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use mockall::predicate::*;
use spl_domain::entities::user::{Role, User};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_register_endpoint_success() {
    let mut mock_repo = MockUserRepository::new();
    let mut mock_role_repo = MockRoleRepository::new();
    let mut mock_encoder = MockPasswordEncoder::new();
    let mut mock_token = MockTokenGenerator::new();

    let new_user_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();

    // Use arbitrary IDs for tests
    let admin_role_id = 3;
    let user_role_id = 1;

    let creator_user = User {
        id: creator_id,
        username: "admin".to_string(),
        email: Some("admin@example.com".to_string()),
        password_hash: "hash".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: admin_role_id,
            name: "admin".to_string(),
            level: 100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        company: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let new_user = User {
        id: new_user_id,
        username: "newuser".to_string(),
        email: Some("new@example.com".to_string()),
        password_hash: "hashed_new".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: 1,
            name: "user".to_string(),
            level: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        company: None, // Simplified for this test as we mocking response
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Middleware Auth flow
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(1)
        .returning(move |_| Ok(serde_json::json!({"sub": creator_id.to_string()})));

    mock_repo
        .expect_get_by_id()
        .with(eq(creator_id))
        .times(1)
        .returning(move |_| Ok(Some(creator_user.clone())));

    // Role Repo Expectations
    // 1. Fetch Creator Role (Admin)
    mock_role_repo
        .expect_get_by_id()
        .with(eq(user_role_id))
        .returning(move |_| {
            Ok(Some(Role {
                id: admin_role_id,
                name: "admin".to_string(),
                level: 100,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    // 2. Fetch Target Role (user)
    mock_role_repo
        .expect_get_by_name()
        .with(eq("user"))
        .returning(move |_| {
            Ok(Some(Role {
                id: user_role_id,
                name: "user".to_string(),
                level: 10,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    // Service Logic
    // 1. Check if user exists (returns None)
    mock_repo
        .expect_get_by_username_and_company()
        .with(eq("newuser"), eq(Some(company_id)))
        .times(1)
        .returning(|_, _| Ok(None));

    // 2. Hash password
    mock_encoder
        .expect_hash()
        .times(1)
        .returning(|_| Ok("hashed_new".to_string()));

    // 3. Create user
    mock_repo
        .expect_create()
        .times(1)
        .returning(move |_| Ok(new_user.clone()));

    let mut mock_company_repo = crate::common::mocks::MockCompanyRepository::new();

    mock_company_repo
        .expect_get_by_id()
        .with(eq(company_id))
        .returning(move |_| {
            Ok(Some(spl_domain::entities::company::Company {
                id: company_id,
                name: "Test Company".to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        });

    let app = build_app(
        mock_repo,
        mock_role_repo,
        mock_company_repo,
        mock_encoder,
        mock_token,
    );

    let register_payload = serde_json::json!({
        "username": "newuser",
        "email": "new@example.com",
        "password": "newpassword123",
        "role": "user",
        "company_id": company_id.to_string()
    });

    let request = Request::builder()
        .uri("/api/v1/auth/register")
        .method("POST")
        .header("Content-Type", "application/json")
        .header("Authorization", "Bearer valid_token")
        .body(Body::from(register_payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body_json["username"], "newuser");
    assert_eq!(body_json["email"], "new@example.com");
    assert_eq!(body_json["id"], new_user_id.to_string());
}


