use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: String,
    #[sea_orm(unique)]
    pub email: String,
    pub password_hash: String,
    #[sea_orm(column_type = "String(StringLen::N(100))")]
    pub name: Option<String>,
    #[sea_orm(column_type = "String(StringLen::N(100))")]
    pub surname: Option<String>,
    pub role_id: i32,
    pub company_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Role,
    #[sea_orm(
        belongs_to = "super::super::company::Entity",
        from = "Column::CompanyId",
        to = "super::super::company::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Company,
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

impl Related<super::super::company::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Company.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
