use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub filepath: String,
    pub prediction_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::adapters::persistence::entities::diagnostics::prediction::Entity",
        from = "Column::PredictionId",
        to = "crate::adapters::persistence::entities::diagnostics::prediction::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Prediction,
}

impl Related<crate::adapters::persistence::entities::diagnostics::prediction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Prediction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
