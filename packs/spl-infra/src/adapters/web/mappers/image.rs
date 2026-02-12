use crate::adapters::web::models::image::ImageResponse;
use spl_domain::entities::image::Image;

impl From<Image> for ImageResponse {
    fn from(image: Image) -> Self {
        Self {
            id: image.id,
            filename: image.filename,
            filepath: image.filepath,
            created_at: image.created_at,
        }
    }
}
