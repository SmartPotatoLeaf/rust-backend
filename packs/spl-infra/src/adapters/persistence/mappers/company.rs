use crate::adapters::persistence::entities::company::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::company::Company;

impl From<Model> for Company {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

impl From<Company> for ActiveModel {
    fn from(entity: Company) -> Self {
        Self {
            id: Set(entity.id),
            name: Set(entity.name),
            description: Set(entity.description),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}

impl From<Company> for Model {
    fn from(entity: Company) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            description: entity.description,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
        }
    }
}
