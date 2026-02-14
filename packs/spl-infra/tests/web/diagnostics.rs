use crate::common::build_app_full;
use crate::common::mocks::*;
use axum::body::to_bytes;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use mockall::predicate::*;
use spl_domain::entities::diagnostics::{Label, MarkType};
use spl_domain::entities::user::{Role, User};
use spl_shared::http::responses::StatusResponse;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_get_all_labels_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_label_repo = MockLabelRepository::new();
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

    let labels = vec![Label {
        id: 1,
        name: "Label 1".to_string(),
        description: Some("Desc 1".to_string()),
        min: 0.0,
        max: 0.5,
        weight: 1,
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

    // Label Mock
    mock_label_repo
        .expect_get_all()
        .times(1)
        .returning(move || Ok(labels.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        mock_label_repo,
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
                .uri("/api/v1/diagnostics/labels")
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
async fn test_get_all_mark_types_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_mark_type_repo = MockMarkTypeRepository::new();
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

    let mark_types = vec![MarkType {
        id: 1,
        name: "Mark 1".to_string(),
        description: Some("Desc 1".to_string()),
        created_at: chrono::Utc::now(),
    }];

    // Auth Mocks
    mock_token
        .expect_validate()
        .with(eq("admin_token"))
        .times(1) // AuthUser
        .returning(move |_| Ok(serde_json::json!({"sub": user_id.to_string()})));

    mock_user_repo
        .expect_get_by_id()
        .with(eq(user_id))
        .times(1)
        .returning(move |_| Ok(Some(admin_user.clone())));

    // MarkType Mock
    mock_mark_type_repo
        .expect_get_all()
        .times(1)
        .returning(move || Ok(mark_types.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        mock_mark_type_repo,
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
                .uri("/api/v1/diagnostics/marks/types")
                .method("GET")
                .header("Authorization", "Bearer admin_token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_label_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_label_repo = MockLabelRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let label_id = 1;

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

    let label = Label {
        id: label_id,
        name: "Label1".to_string(),
        description: Some("Desc 1".to_string()),
        min: 0.0,
        max: 0.5,
        weight: 1,
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

    // Label Mock
    let label_clone = label.clone();
    mock_label_repo
        .expect_get_by_id()
        .with(eq(label_id))
        .times(1)
        .returning(move |_| Ok(Some(label_clone.clone())));

    mock_label_repo
        .expect_update()
        .times(1)
        .returning(move |_| Ok(label.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        mock_label_repo,
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        MockPredictionRepository::new(),
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let payload = serde_json::json!({
        "name": "UpdatedLabel",
        "weight": 2
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/diagnostics/labels/{}", label_id))
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
async fn test_delete_label_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_label_repo = MockLabelRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let label_id = 1;

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

    let label = Label {
        id: label_id,
        name: "Label1".to_string(),
        description: Some("Desc 1".to_string()),
        min: 0.0,
        max: 0.5,
        weight: 1,
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

    // Label Mock
    // LabelService::delete does not fetch first.

    mock_label_repo
        .expect_delete()
        .with(eq(label_id))
        .times(1)
        .returning(move |_| Ok(label.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        mock_label_repo,
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
                .uri(format!("/api/v1/diagnostics/labels/{}", label_id))
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

#[tokio::test]
async fn test_delete_prediction_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_prediction_repo = MockPredictionRepository::new();
    let mut mock_token = MockTokenGenerator::new();

    let user_id = Uuid::new_v4();
    let prediction_id = Uuid::new_v4();

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

    use spl_domain::entities::image::Image;
    let image = Image {
        id: Uuid::new_v4(),
        user_id,
        filename: "image.jpg".to_string(),
        filepath: "path/to/image.jpg".to_string(),
        created_at: chrono::Utc::now(),
        prediction_id: Some(prediction_id),
    };

    use spl_domain::entities::diagnostics::Label;
    let label = Label {
        id: 1,
        name: "Label 1".to_string(),
        description: Some("Desc 1".to_string()),
        min: 0.0,
        max: 0.5,
        weight: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    use spl_domain::entities::diagnostics::Prediction;
    let prediction = Prediction {
        id: prediction_id,
        user: user.clone(),
        image: image.clone(),
        label: label.clone(),
        plot_id: None,
        presence_confidence: 0.8,
        absence_confidence: 0.2,
        severity: 50.0,
        created_at: chrono::Utc::now(),
        marks: vec![],
        feedback: None
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

    // Prediction Mock
    let prediction_clone = prediction.clone();
    mock_prediction_repo
        .expect_get_by_user_id_and_id()
        .with(eq(user_id), eq(prediction_id))
        .times(1)
        .returning(move |_, _| Ok(Some(prediction_clone.clone())));

    mock_prediction_repo
        .expect_delete()
        .with(eq(prediction_id))
        .times(1)
        .returning(move |_| Ok(prediction.clone()));

    let app = build_app_full(
        mock_user_repo,
        MockRoleRepository::new(),
        MockCompanyRepository::new(),
        MockRecommendationRepository::new(),
        MockRecommendationCategoryRepository::new(),
        MockLabelRepository::new(),
        MockMarkTypeRepository::new(),
        MockPlotRepository::new(),
        mock_prediction_repo,
        MockPredictionMarkRepository::new(),
        MockImageRepository::new(),
        MockPasswordEncoder::new(),
        mock_token,
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/diagnostics/predictions/{}", prediction_id))
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


