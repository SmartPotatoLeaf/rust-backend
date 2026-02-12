use crate::adapters::persistence::entities::{feedback, image, plot, user};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "predictions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub image_id: Uuid,
    pub label_id: i32,
    pub plot_id: Option<Uuid>,
    pub presence_confidence: f32,
    pub absence_confidence: f32,
    pub severity: f32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::label::Entity",
        from = "Column::LabelId",
        to = "super::label::Column::Id"
    )]
    Label,
    #[sea_orm(
        belongs_to = "user::Entity",
        from = "Column::UserId",
        to = "user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
    #[sea_orm(
        belongs_to = "image::Entity",
        from = "Column::ImageId",
        to = "image::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Image,
    #[sea_orm(has_many = "super::prediction_mark::Entity")]
    PredictionMark,
    #[sea_orm(
        belongs_to = "plot::Entity",
        from = "Column::PlotId",
        to = "plot::Column::Id"
    )]
    Plot,
    #[sea_orm(
        has_one = "feedback::Entity",
        from = "Column::Id",
        to = "feedback::Column::PredictionId"
    )]
    Feedback
}

impl Related<super::label::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Label.def()
    }
}

impl Related<user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<image::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Image.def()
    }
}

impl Related<super::prediction_mark::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PredictionMark.def()
    }
}

impl Related<plot::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Plot.def()
    }
}

impl Related<feedback::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Feedback.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
