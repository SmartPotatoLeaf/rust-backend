use crate::adapters::persistence::entities::diagnostics::label::{ActiveModel, Model};
use spl_domain::entities::diagnostics::Label;
use spl_shared::{map_mirror, maps_set};

map_mirror!(
    Model, Label {
        id, name, description, min, max, weight,
        #into [ created_at, updated_at ]
    }
);

maps_set!(
    ActiveModel {
        id, name, description, min, max, weight,
        #into [ created_at, updated_at ]
    } #from [ Label ]
);
