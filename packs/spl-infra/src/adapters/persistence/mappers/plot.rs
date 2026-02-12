use crate::adapters::persistence::entities::plot::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::plot::Plot;

impl From<Model> for Plot {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            company_id: model.company_id,
            name: model.name,
            description: model.description,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

impl From<Plot> for ActiveModel {
    fn from(entity: Plot) -> Self {
        Self {
            id: Set(entity.id),
            company_id: Set(entity.company_id),
            name: Set(entity.name),
            description: Set(entity.description),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}

impl From<Plot> for Model {
    fn from(entity: Plot) -> Self {
        Self {
            id: entity.id,
            company_id: entity.company_id,
            name: entity.name,
            description: entity.description,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
        }
    }
}
