use crate::adapters::persistence::entities::company::{ActiveModel, Model};
use spl_domain::entities::company::Company;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, Company {
   id, name, description,
    #into [ created_at, updated_at ]
});

maps_set!(ActiveModel {
    id, name, description,
    #into [ created_at, updated_at ]
  } #from [ Company ]
);
