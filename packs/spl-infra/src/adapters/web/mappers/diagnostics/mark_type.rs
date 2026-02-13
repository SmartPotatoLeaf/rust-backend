use crate::adapters::web::models::diagnostics::{
    CreateMarkTypeRequest, MarkTypeResponse, SimplifiedMarkTypeResponse, UpdateMarkTypeRequest,
};
use spl_application::dtos::diagnostics::{CreateMarkTypeDto, UpdateMarkTypeDto};
use spl_domain::entities::diagnostics::MarkType;
use spl_shared::{map_mirror, maps_to};

map_mirror!(
    CreateMarkTypeRequest,
    CreateMarkTypeDto {
        name,
        description,
    }
);

map_mirror!(
    UpdateMarkTypeRequest,
    UpdateMarkTypeDto {
        name,
        description,
    }
);

map_mirror!(
    MarkType,
    MarkTypeResponse {
        id,
        name,
        description,
        created_at,
    }
);

maps_to!(SimplifiedMarkTypeResponse {
    id,
    name,
} #from [ MarkType ]);
