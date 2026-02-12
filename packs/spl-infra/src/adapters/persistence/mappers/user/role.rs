use crate::adapters::persistence::entities::user::role::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::user::Role;

impl From<Model> for Role {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            level: model.level,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}

impl From<Role> for Model {
    fn from(entity: Role) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            level: entity.level,
            created_at: entity.created_at.into(),
            updated_at: entity.updated_at.into(),
        }
    }
}

impl From<Role> for ActiveModel {
    fn from(entity: Role) -> Self {
        Self {
            id: Set(entity.id), // No autoincrement check for now, trusting input or db default if handled elsewhere, but id is i32 here.
            name: Set(entity.name),
            level: Set(entity.level),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}
