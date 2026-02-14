use crate::adapters::web::models::image::{ImageResponse, RawImageResponse};
use spl_domain::entities::image::{Image, RawImage};
use spl_shared::map_mirror;
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

map_mirror!(
    Image,
    ImageResponse {
        id,
        user_id,
        filename,
        filepath,
        prediction_id,
        created_at,
    }
);

impl From<RawImage> for RawImageResponse {
    fn from(raw_image: RawImage) -> Self {
        let data = BASE64.encode(&raw_image.data);
        Self {
            filename: raw_image.filename,
            data,
        }
    }
}