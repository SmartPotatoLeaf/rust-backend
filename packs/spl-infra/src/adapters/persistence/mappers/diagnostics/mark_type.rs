use crate::adapters::persistence::entities::diagnostics::mark_type::{ActiveModel, Model};
use spl_domain::entities::diagnostics::MarkType;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, MarkType {
    id,
    name,
    description,
    #into [ created_at ]
});

maps_set!(
    ActiveModel {
        id,
        name,
        description,
        #into [ created_at ]
    } #from [ MarkType ]
);
