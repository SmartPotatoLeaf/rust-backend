use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use mockall::predicate::*;
use spl_domain::entities::user::{Role, User};
use tower::ServiceExt;

mod common;
use common::*;

#[tokio::test]
async fn test_rbac_admin_access_user_route() {
    // 1. Setup Mocks
    let mut mock_repo = MockUserRepository::new();
    let mut mock_role_repo = MockRoleRepository::new();
    let mock_encoder = MockPasswordEncoder::new();
    let mut mock_token = MockTokenGenerator::new();

    // 2. Define Data
    let user_id = uuid::Uuid::new_v4();
    let admin_role = Role {
        id: 1,
        name: "admin".to_string(),
        level: 100,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let required_role = Role {
        id: 2,
        name: "user".to_string(), // The route will require "user"
        level: 10,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // UserRepo: When middleware (AuthUser) looks up user by ID from token
    let user = User {
        id: user_id,
        username: "test_admin".to_string(),
        email: Some("admin@example.com".to_string()),
        password_hash: "hashed_secret".to_string(),
        name: None,
        surname: None,
        role: admin_role.clone(), // Has Admin Role object
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // 3. Configure Mocks
    // RoleRepo: Return admin role when asked for user's role_id (1)
    mock_role_repo
        .expect_get_by_id()
        .with(eq(1))
        .returning(move |_| Ok(Some(admin_role.clone())));

    // RoleRepo: Return "user" role when asked for the required role definition
    // The middleware calls `get_by_name("user")`
    mock_role_repo
        .expect_get_by_name()
        .with(eq("user"))
        .returning(move |_| Ok(Some(required_role.clone())));

    // Token: Validate returns generic payload
    mock_token
        .expect_validate()
        .returning(move |_| Ok(serde_json::json!({ "sub": user_id.to_string(), "role": "admin" })));

    mock_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .returning(move |_| Ok(Some(user.clone())));

    // 4. Build App
    let mock_company_repo = MockCompanyRepository::new();
    let app = build_app(
        mock_repo,
        mock_role_repo,
        mock_company_repo,
        mock_encoder,
        mock_token,
    );

    // 5. Execute Request
    // The /api/v1/users/me endpoint is protected by:
    // 1. AuthUser (Checks extraction)
    // 2. PermissionCheck (Checks Role) -> Requires "user"
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/users/me")
                .header("Authorization", "Bearer valild_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 6. Assert
    // Admin (100) >= User (10) -> Should be OK (200)
    assert_eq!(response.status(), StatusCode::OK);
}
