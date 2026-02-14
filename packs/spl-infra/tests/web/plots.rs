use crate::common::build_app_full;
use crate::common::mocks::*;
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mockall::predicate::*;
use spl_domain::entities::company::Company;
use spl_domain::entities::plot::Plot;
use spl_domain::entities::user::{Role, User};
use spl_domain::ports::repositories::plot::DetailedPlot;
use spl_shared::http::responses::StatusResponse;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_get_plots_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
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
        company: Some(company),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let plot_id = Uuid::new_v4();
    let plots = vec![Plot {
        id: plot_id,
        name: "Plot 1".to_string(),
        description: Some("Desc 1".to_string()),
        company_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Plot Mock
    mock_plot_repo
        .expect_get_by_company_id()
        .with(eq(company_id))
        .times(1)
        .returning(move |_| Ok(plots.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plots")
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
async fn test_get_plot_by_id_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();
    let plot_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
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
        company: Some(company),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let plot = Plot {
        id: plot_id,
        name: "Plot 1".to_string(),
        description: Some("Desc 1".to_string()),
        company_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Plot Mock
    mock_plot_repo
        .expect_get_by_company_id_and_id()
        .with(eq(company_id), eq(plot_id))
        .times(1)
        .returning(move |_, _| Ok(Some(plot.clone())));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/plots/{}", plot_id))
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
async fn test_create_plot_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();
    let plot_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let user = User {
        id: user_id,
        username: "supervisor".to_string(),
        email: "supervisor@example.com".to_string(),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: 3,
            name: "supervisor".to_string(),
            level: 50, // Supervisor level
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(company),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let plot = Plot {
        id: plot_id,
        name: "New Plot".to_string(),
        description: Some("New Desc".to_string()),
        company_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Plot Mock
    let plot_clone = plot.clone();
    mock_plot_repo
        .expect_create()
        .times(1)
        .returning(move |_| Ok(plot_clone.clone()));

    // We expect the company methods to be called for validation if any,
    // but create mainly just saves.

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "New Plot",
        "description": "New Desc"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plots")
                .method("POST")
                .header("Authorization", "Bearer valid_token")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_plot_forbidden() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let user = User {
        id: user_id,
        username: "user".to_string(),
        email: "user@example.com".to_string(),
        password_hash: "hashed".to_string(),
        name: None,
        surname: None,
        role: Role {
            id: 2,
            name: "user".to_string(),
            level: 10, // Insufficient level
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(company),
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

    // No calls to plot repo expected

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "New Plot",
        "description": "New Desc"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plots")
                .method("POST")
                .header("Authorization", "Bearer valid_token")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_plot_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();
    let plot_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
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
            name: "supervisor".to_string(),
            level: 10,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(company),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let plot = Plot {
        id: plot_id,
        name: "Plot 1".to_string(),
        description: Some("Desc 1".to_string()),
        company_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Plot Mock
    let plot_clone = plot.clone();
    mock_plot_repo
        .expect_get_by_company_id_and_id()
        .with(eq(company_id), eq(plot_id))
        .times(1)
        .returning(move |_, _| Ok(Some(plot_clone.clone())));

    mock_plot_repo
        .expect_update()
        .times(1)
        .returning(move |_| Ok(plot.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "Updated Plot"
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/plots/{}", plot_id))
                .method("PUT")
                .header("Authorization", "Bearer valid_token")
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
async fn test_delete_plot_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();
    let plot_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
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
            name: "supervisor".to_string(),
            level: 10,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        company: Some(company),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let plot = Plot {
        id: plot_id,
        name: "Plot 1".to_string(),
        description: Some("Desc 1".to_string()),
        company_id,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Plot Mock
    let plot_clone = plot.clone();
    mock_plot_repo
        .expect_get_by_company_id_and_id()
        .with(eq(company_id), eq(plot_id))
        .times(1)
        .returning(move |_, _| Ok(Some(plot_clone.clone())));

    mock_plot_repo
        .expect_delete()
        .with(eq(plot_id))
        .times(1)
        .returning(move |_| Ok(plot.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/plots/{}", plot_id))
                .method("DELETE")
                .header("Authorization", "Bearer valid_token")
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

#[tokio::test]
async fn test_detailed_plots_with_label_filtering() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_plot_repo = MockPlotRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let company_id = Uuid::new_v4();
    let plot_id = Uuid::new_v4();

    let company = Company {
        id: company_id,
        name: "Test Company".to_string(),
        description: None,
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
        company: Some(company),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let detailed_plot = DetailedPlot {
        id: Some(plot_id),
        name: "Filtered Plot".to_string(),
        description: Some("Filtered Desc".to_string()),
        created_at: chrono::Utc::now(),
        total_diagnosis: 10,
        last_diagnosis: Some(chrono::Utc::now()),
        matching_diagnosis: 5,
    };

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("valid_token"))
        .times(2)
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(2)
        .returning(move |_| Ok(Some(user.clone())));

    // Plot Mock for filtered request
    let labels_filter = vec!["Rust".to_string(), "Fungus".to_string()];

    let mut mock_prediction_repo = MockPredictionRepository::new();
    mock_prediction_repo
        .expect_has_unassigned_predictions()
        .with(eq(user_id))
        .returning(|_| Ok(false));

    mock_plot_repo
        .expect_get_detailed()
        .with(eq(company_id), eq(0), eq(10), eq(labels_filter.clone())) // Use labels_clone here
        .times(1)
        .returning(move |_, _, _, _| Ok((1, vec![detailed_plot.clone()])));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        mock_plot_repo,
        mock_prediction_repo,
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "page": 1,
        "limit": 10,
        "labels": ["Rust", "Fungus"]
    });

    // Query with labels
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/plots/detailed")
                .method("POST")
                .header("Authorization", "Bearer valid_token")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}


