use crate::adapters::web::models::diagnostics::{
    CreateLabelRequest, LabelResponse, SimplifiedLabelResponse, UpdateLabelRequest,
};
use spl_application::dtos::diagnostics::{CreateLabelDto, UpdateLabelDto};
use spl_domain::entities::diagnostics::Label;

impl From<CreateLabelRequest> for CreateLabelDto {
    fn from(req: CreateLabelRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
            min: req.min,
            max: req.max,
            weight: req.weight,
        }
    }
}

impl From<UpdateLabelRequest> for UpdateLabelDto {
    fn from(req: UpdateLabelRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
            min: req.min,
            max: req.max,
            weight: req.weight,
        }
    }
}

impl From<Label> for LabelResponse {
    fn from(label: Label) -> Self {
        Self {
            id: label.id,
            name: label.name,
            description: label.description,
            min: label.min,
            max: label.max,
            weight: label.weight,
            created_at: label.created_at,
            updated_at: label.updated_at,
        }
    }
}

impl From<Label> for SimplifiedLabelResponse {
    fn from(label: Label) -> Self {
        Self {
            id: label.id,
            name: label.name,
        }
    }
}
