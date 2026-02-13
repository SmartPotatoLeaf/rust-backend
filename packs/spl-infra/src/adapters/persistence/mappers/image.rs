use crate::adapters::persistence::entities::image::{ActiveModel, Model};
use spl_domain::entities::image::Image;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, Image {
   id, user_id, filename, filepath, prediction_id,
   #into [ created_at ]
});

maps_set!(ActiveModel {
   id, user_id, filename, filepath, prediction_id,
   #into [ created_at ]
} #from [ Image ]);
