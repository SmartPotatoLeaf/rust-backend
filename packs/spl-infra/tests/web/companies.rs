use crate::common::build_app;
use crate::common::mocks::{
    MockCompanyRepository, MockPasswordEncoder, MockRoleRepository, MockTokenGenerator,
    MockUserRepository,
};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use mockall::predicate::*;
use spl_domain::entities::company::Company;
use spl_domain::entities::user::{Role, User};
use spl_shared::http::responses::StatusResponse;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_get_company_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_company_repo = MockCompanyRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let company_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: Some("Description".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let user = User {
        id: user_id,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
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
        company: Some(company.clone()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
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

    // Company Mock
    mock_company_repo
        .expect_get_by_id()
        .with(eq(company_id))
        .times(1)
        .returning(move |_| Ok(Some(company.clone())));

    let app = build_app(
        mock_user_repo,
        MockRoleRepository::new(),
        mock_company_repo,
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/companies/{}", company_id))
                .method("GET")
                .header("Authorization", "Bearer valid_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_company_admin_only_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_company_repo = MockCompanyRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();

    let admin_user = User {
        id: user_id,
        username: "adminuser".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: 1,
            name: "admin".to_string(),
            level: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("admin_token"))
        .times(2) // Once for AuthUser, once for permission_check middleware
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2) // Once for AuthUser, once for permission_check middleware
        .returning(move |_| Ok(Some(admin_user.clone())));

    // Company Mock
    mock_company_repo
        .expect_create()
        .times(1)
        .returning(|c| Ok(c));

    let app = build_app(
        mock_user_repo,
        MockRoleRepository::new(),
        mock_company_repo,
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "New Company",
        "description": "New Description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/companies")
                .method("POST")
                .header("Authorization", "Bearer admin_token")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_update_company_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_company_repo = MockCompanyRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();

    let admin_user = User {
        id: user_id,
        username: "adminuser".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: 1,
            name: "admin".to_string(),
            level: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: Some("Description".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("admin_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(admin_user.clone())));

    // Company Mock
    let company_for_get = company.clone();
    mock_company_repo
        .expect_get_by_id()
        .with(eq(company_id))
        .times(1)
        .returning(move |_| Ok(Some(company_for_get.clone())));

    mock_company_repo
        .expect_update()
        .times(1)
        .returning(move |_| Ok(company.clone()));

    let app = build_app(
        mock_user_repo,
        MockRoleRepository::new(),
        mock_company_repo,
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "Updated Company",
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/companies/{}", company_id))
                .method("PUT")
                .header("Authorization", "Bearer admin_token")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let status_response: StatusResponse = serde_json::from_slice(&body).unwrap();
    assert!(status_response.success);
}

#[tokio::test]
async fn test_delete_company_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_company_repo = MockCompanyRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();

    let admin_user = User {
        id: user_id,
        username: "adminuser".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: 1,
            name: "admin".to_string(),
            level: 100,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: Some("Description".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("admin_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(admin_user.clone())));

    // Company Mock
    mock_company_repo
        .expect_delete()
        .times(1)
        .returning(move |_| Ok(company.clone()));

    let app = build_app(
        mock_user_repo,
        MockRoleRepository::new(),
        mock_company_repo,
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/companies/{}", company_id))
                .method("DELETE")
                .header("Authorization", "Bearer admin_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let status_response: StatusResponse = serde_json::from_slice(&body).unwrap();
    assert!(status_response.success);
}


