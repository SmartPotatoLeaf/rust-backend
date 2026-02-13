use crate::adapters::web::models::diagnostics::{
    CreateLabelRequest, LabelResponse, SimplifiedLabelResponse, UpdateLabelRequest,
};
use spl_application::dtos::diagnostics::{CreateLabelDto, UpdateLabelDto};
use spl_domain::entities::diagnostics::Label;
use spl_shared::{map_mirror, maps_to};

map_mirror!(
    CreateLabelRequest,
    CreateLabelDto {
        name,
        description,
        min,
        max,
        weight,
    }
);

map_mirror!(
    UpdateLabelRequest,
    UpdateLabelDto {
        name,
        description,
        min,
        max,
        weight,
    }
);

map_mirror!(
    Label,
    LabelResponse {
        id,
        name,
        description,
        min,
        max,
        weight,
        created_at,
        updated_at,
    }
);

maps_to!(SimplifiedLabelResponse {
    id,
    name,
} #from [ Label ]);
