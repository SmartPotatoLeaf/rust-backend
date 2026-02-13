use crate::adapters::persistence::entities::user::role::{ActiveModel, Model};
use spl_domain::entities::user::Role;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, Role {
    id,
    name,
    level,
    #into [ created_at, updated_at ]
});

maps_set!(ActiveModel {
    id,
    name,
    level,
    #into [ created_at, updated_at ]
} #from [ Role ]);
