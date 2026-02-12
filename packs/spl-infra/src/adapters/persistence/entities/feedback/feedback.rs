use sea_orm::entity::prelude::*;

use super::status;
use crate::adapters::persistence::entities::diagnostics::{label, prediction};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "feedbacks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text", nullable)]
    pub comment: Option<String>,
    pub status_id: i32,
    pub correct_label_id: Option<i32>,
    #[sea_orm(unique)]
    pub prediction_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "status::Entity",
        from = "Column::StatusId",
        to = "status::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    FeedbackStatus,
    #[sea_orm(
        belongs_to = "label::Entity",
        from = "Column::CorrectLabelId",
        to = "label::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Label,
    #[sea_orm(
        belongs_to = "prediction::Entity",
        from = "Column::PredictionId",
        to = "prediction::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Prediction,
}

impl Related<status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FeedbackStatus.def()
    }
}

impl Related<label::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Label.def()
    }
}

impl Related<prediction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prediction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
