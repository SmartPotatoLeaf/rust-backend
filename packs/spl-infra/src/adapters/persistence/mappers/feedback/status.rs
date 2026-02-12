use sea_orm::Set;
use spl_domain::entities::feedback::FeedbackStatus;

use crate::adapters::persistence::entities::feedback::status::{ActiveModel, Model};

impl From<Model> for FeedbackStatus {
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

impl From<FeedbackStatus> for ActiveModel {
    fn from(entity: FeedbackStatus) -> Self {
        Self {
            id: Set(entity.id),
            name: Set(entity.name),
            description: Set(entity.description),
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}
