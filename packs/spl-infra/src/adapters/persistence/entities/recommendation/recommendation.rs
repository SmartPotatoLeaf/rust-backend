use sea_orm::entity::prelude::*;

use super::category;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recommendations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text", nullable)]
    pub description: Option<String>,
    pub category_id: i32,
    #[sea_orm(column_type = "Float")]
    pub min_severity: f32,
    #[sea_orm(column_type = "Float")]
    pub max_severity: f32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "category::Entity",
        from = "Column::CategoryId",
        to = "category::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    RecommendationCategory,
}

impl Related<category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RecommendationCategory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
