use crate::adapters::persistence::entities::diagnostics::mark_type::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::diagnostics::MarkType;

impl From<Model> for MarkType {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            created_at: model.created_at.into(),
        }
    }
}

impl From<MarkType> for ActiveModel {
    fn from(entity: MarkType) -> Self {
        Self {
            id: Set(entity.id),
            name: Set(entity.name),
            description: Set(entity.description),
            created_at: Set(entity.created_at.into()),
        }
    }
}

impl From<MarkType> for Model {
    fn from(entity: MarkType) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            description: entity.description,
            created_at: entity.created_at.into(),
        }
    }
}
