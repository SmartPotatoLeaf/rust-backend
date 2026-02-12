use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use spl_application::dtos::image::CreateImageDto;
use spl_application::services::image::ImageService;
use spl_domain::entities::image::Image;
use spl_domain::ports::repositories::image::ImageRepository;
use spl_shared::error::Result;
use std::sync::Arc;
use uuid::Uuid;

// Mock definitions
mock! {
    pub ImageRepository {}
    #[async_trait]
    impl ImageRepository for ImageRepository {
        async fn create(&self, image: Image) -> Result<Image>;
        async fn get_by_id(&self, id: Uuid) -> Result<Option<Image>>;
        async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<Image>>;
        async fn update(&self, image: Image) -> Result<Image>;
        async fn delete(&self, id: Uuid) -> Result<()>;
    }
}

#[tokio::test]
async fn test_create_image() {
    let mut mock_repo = MockImageRepository::new();

    let user_id = Uuid::new_v4();
    let dto = CreateImageDto {
        filename: "test.jpg".to_string(),
        filepath: "/tmp/test.jpg".to_string(),
        user_id,
        prediction_id: None,
    };

    // Expect create to be called once
    mock_repo
        .expect_create()
        .with(function(move |img: &Image| {
            img.filename == "test.jpg" && img.user_id == user_id
        }))
        .times(1)
        .returning(|img| Ok(img));

    let service = ImageService::new(Arc::new(mock_repo));

    let result = service.create(dto).await;
    assert!(result.is_ok());
    let image = result.unwrap();
    assert_eq!(image.filename, "test.jpg");
    assert_eq!(image.user_id, user_id);
}

#[tokio::test]
async fn test_get_image_by_id() {
    let mut mock_repo = MockImageRepository::new();
    let image_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let image = Image {
        id: image_id,
        user_id,
        filename: "test.jpg".to_string(),
        filepath: "/tmp/test.jpg".to_string(),
        prediction_id: None,
        created_at: chrono::Utc::now(),
    };

    mock_repo
        .expect_get_by_id()
        .with(eq(image_id))
        .times(1)
        .returning(move |_| Ok(Some(image.clone())));

    let service = ImageService::new(Arc::new(mock_repo));

    let result = service.get_by_id(image_id).await;
    assert!(result.is_ok());
    let fetched = result.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().id, image_id);
}
