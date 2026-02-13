use crate::adapters::persistence::entities::feedback::status::{ActiveModel, Model};
use spl_domain::entities::feedback::FeedbackStatus;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, FeedbackStatus {
    id,
    name,
    description,
    #into [ created_at, updated_at]
});

maps_set!(ActiveModel {
    id, name, description,
    #into [ created_at, updated_at]
} #from [FeedbackStatus]);
