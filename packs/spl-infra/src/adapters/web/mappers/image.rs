use crate::adapters::web::models::image::ImageResponse;
use spl_domain::entities::image::Image;
use spl_shared::map_mirror;

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
