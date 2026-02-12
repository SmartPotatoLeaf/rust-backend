use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "prediction_marks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: serde_json::Value,
    pub prediction_id: Uuid,
    pub mark_type_id: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::mark_type::Entity",
        from = "Column::MarkTypeId",
        to = "super::mark_type::Column::Id"
    )]
    MarkType,
    #[sea_orm(
        belongs_to = "super::prediction::Entity",
        from = "Column::PredictionId",
        to = "super::prediction::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Prediction,
}

impl Related<super::mark_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MarkType.def()
    }
}

impl Related<super::prediction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prediction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
