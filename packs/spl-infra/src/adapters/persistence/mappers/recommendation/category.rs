use crate::adapters::persistence::entities::recommendation::category::{ActiveModel, Model};
use spl_domain::entities::recommendation::Category;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, Category {
    id,
    name,
    description,
    #into [ created_at, updated_at]
});

maps_set!(ActiveModel {
    id,
    name,
    description,
    #into [ created_at, updated_at]
} #from [ Category ]
);
