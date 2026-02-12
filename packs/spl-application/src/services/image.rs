use std::sync::Arc;
use uuid::Uuid;

use spl_domain::entities::image::Image;
use spl_domain::ports::repositories::image::ImageRepository;
use spl_shared::error::Result;

use crate::dtos::image::CreateImageDto;

pub struct ImageService {
    image_repo: Arc<dyn ImageRepository>,
}

impl ImageService {
    pub fn new(image_repo: Arc<dyn ImageRepository>) -> Self {
        Self { image_repo }
    }

    pub async fn create(&self, dto: CreateImageDto) -> Result<Image> {
        let image = Image {
            id: Uuid::new_v4(),
            user_id: dto.user_id,
            filename: dto.filename,
            filepath: dto.filepath,
            prediction_id: dto.prediction_id,
            created_at: chrono::Utc::now(),
        };

        self.image_repo.create(image).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Image>> {
        self.image_repo.get_by_id(id).await
    }

    pub async fn update(&self, image: Image) -> Result<Image> {
        self.image_repo.update(image).await
    }
}
