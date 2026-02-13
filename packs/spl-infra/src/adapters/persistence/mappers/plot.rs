use crate::adapters::persistence::entities::plot::{ActiveModel, Model};
use spl_domain::entities::plot::Plot;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, Plot {
   id, company_id, name, description,
   #into [ created_at, updated_at ]
});

maps_set!(ActiveModel {
  id, company_id, name, description,
  #into [ created_at, updated_at ]
} #from [ Plot ]);
