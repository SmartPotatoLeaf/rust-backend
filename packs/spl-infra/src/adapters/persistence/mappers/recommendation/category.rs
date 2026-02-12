use sea_orm::Set;
use spl_domain::entities::recommendation::Category;

use crate::adapters::persistence::entities::recommendation::category::{ActiveModel, Model};

impl From<Model> for Category {
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

impl From<Category> for ActiveModel {
    fn from(entity: Category) -> Self {
        Self {
            id: Set(entity.id),
            name: Set(entity.name),
            description: Set(entity.description),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}
