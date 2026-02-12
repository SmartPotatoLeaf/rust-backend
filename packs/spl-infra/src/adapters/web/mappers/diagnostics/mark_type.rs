use crate::adapters::web::models::diagnostics::{
    CreateMarkTypeRequest, MarkTypeResponse, SimplifiedMarkTypeResponse, UpdateMarkTypeRequest,
};
use spl_application::dtos::diagnostics::{CreateMarkTypeDto, UpdateMarkTypeDto};
use spl_domain::entities::diagnostics::MarkType;

impl From<CreateMarkTypeRequest> for CreateMarkTypeDto {
    fn from(req: CreateMarkTypeRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<UpdateMarkTypeRequest> for UpdateMarkTypeDto {
    fn from(req: UpdateMarkTypeRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<MarkType> for MarkTypeResponse {
    fn from(mark_type: MarkType) -> Self {
        Self {
            id: mark_type.id,
            name: mark_type.name,
            description: mark_type.description,
            created_at: mark_type.created_at.to_rfc3339(),
        }
    }
}

impl From<MarkType> for SimplifiedMarkTypeResponse {
    fn from(mark_type: MarkType) -> Self {
        Self {
            id: mark_type.id,
            name: mark_type.name,
        }
    }
}
