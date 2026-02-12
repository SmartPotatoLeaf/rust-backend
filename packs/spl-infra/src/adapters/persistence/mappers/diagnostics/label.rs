use crate::adapters::persistence::entities::diagnostics::label::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::diagnostics::Label;

impl From<Model> for Label {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            min: model.min,
            max: model.max,
            weight: model.weight,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

impl From<Label> for ActiveModel {
    fn from(entity: Label) -> Self {
        Self {
            id: Set(entity.id),
            name: Set(entity.name),
            description: Set(entity.description),
            min: Set(entity.min),
            max: Set(entity.max),
            weight: Set(entity.weight),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}

impl From<Label> for Model {
    fn from(entity: Label) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            description: entity.description,
            min: entity.min,
            max: entity.max,
            weight: entity.weight,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
        }
    }
}
