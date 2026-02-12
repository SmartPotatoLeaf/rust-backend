use crate::adapters::persistence::entities::image::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::image::Image;

impl From<Model> for Image {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            filename: model.filename,
            filepath: model.filepath,
            prediction_id: model.prediction_id,
            created_at: model.created_at.into(),
        }
    }
}

impl From<Image> for ActiveModel {
    fn from(entity: Image) -> Self {
        Self {
            id: Set(entity.id),
            user_id: Set(entity.user_id),
            filename: Set(entity.filename),
            filepath: Set(entity.filepath),
            prediction_id: Set(entity.prediction_id),
            created_at: Set(entity.created_at.into()),
        }
    }
}

impl From<Image> for Model {
    fn from(entity: Image) -> Self {
        Self {
            id: entity.id,
            user_id: entity.user_id,
            filename: entity.filename,
            filepath: entity.filepath,
            prediction_id: entity.prediction_id,
            created_at: entity.created_at.into(),
        }
    }
}
