use crate::common::build_app_full;
use crate::common::mocks::*;
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mockall::predicate::*;
use spl_domain::entities::recommendation::{Category, Recommendation};
use spl_domain::entities::user::{Role, User};
use spl_shared::http::responses::StatusResponse;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_get_all_categories_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_rec_cat_repo = MockRecommendationCategoryRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
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
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let categories = vec![Category {
        id: 1,
        name: "Cat 1".to_string(),
        description: Some("Desc 1".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

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

    // Category Mock
    mock_rec_cat_repo
        .expect_get_all()
        .times(1)
        .returning(move || Ok(categories.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        mock_rec_cat_repo,
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/recommendation/categories")
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
async fn test_get_all_recommendations_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_rec_repo = MockRecommendationRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
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
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let categories = vec![Category {
        id: 1,
        name: "Cat 1".to_string(),
        description: Some("Desc 1".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

    let recommendations = vec![Recommendation {
        id: Uuid::new_v4(),
        description: Some("Rec 1".to_string()),
        min_severity: 0.0,
        max_severity: 0.5,
        category: categories[0].clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

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

    // Recommendation Mock
    mock_rec_repo
        .expect_get_all()
        .times(1)
        .returning(move || Ok(recommendations.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        mock_rec_repo,
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/recommendations")
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
async fn test_update_category_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_rec_cat_repo = MockRecommendationCategoryRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let category_id = 1;

    let user = User {
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

    let category = Category {
        id: category_id,
        name: "Cat 1".to_string(),
        description: Some("Desc 1".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("admin_token")) // Admin token required for update
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Category Mock
    let category_clone = category.clone();
    mock_rec_cat_repo
        .expect_get_by_id()
        .with(eq(category_id))
        .times(1)
        .returning(move |_| Ok(Some(category_clone.clone())));

    mock_rec_cat_repo
        .expect_update()
        .times(1)
        .returning(move |_| Ok(category.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        mock_rec_cat_repo,
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "Updated Cat",
        "description": "Updated Desc"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/recommendation/categories/{}", category_id))
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
async fn test_delete_category_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_rec_cat_repo = MockRecommendationCategoryRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let category_id = 1;

    let user = User {
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

    let category = Category {
        id: category_id,
        name: "Cat 1".to_string(),
        description: Some("Desc 1".to_string()),
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
        .returning(move |_| Ok(Some(user.clone())));

    // Category Mock
    // CategoryService::delete does not fetch first.

    mock_rec_cat_repo
        .expect_delete()
        .with(eq(category_id))
        .times(1)
        .returning(move |_| Ok(category.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        mock_rec_cat_repo,
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/recommendation/categories/{}", category_id))
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


