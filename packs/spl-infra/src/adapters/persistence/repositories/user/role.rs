use crate::adapters::persistence::entities::user::role;
use async_trait::async_trait;
use sea_orm::*;
use spl_domain::entities::user::Role;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::user::RoleRepository;
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};

pub struct DbRoleRepository {
    db: DatabaseConnection,
}

impl DbRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CrudRepository<Role, i32> for DbRoleRepository {
    async fn get_by_id(&self, id: i32) -> Result<Option<Role>> {
        crud::get_by_id::<role::Entity, Role, i32>(&self.db, id).await
    }

    async fn create(&self, entity: Role) -> Result<Role> {
        crud::create::<role::Entity, Role>(&self.db, entity).await
    }

    async fn update(&self, entity: Role) -> Result<Role> {
        crud::update::<role::Entity, Role>(&self.db, entity).await
    }

    async fn delete(&self, id: i32) -> Result<Role> {
        crud::delete::<role::Entity, Role, i32>(&self.db, id).await
    }
}

#[async_trait]
impl RoleRepository for DbRoleRepository {
    async fn get_by_name(&self, name: &str) -> Result<Option<Role>> {
        let model = role::Entity::find()
            .filter(role::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(model.map(|m| m.into()))
    }

    async fn get_all(&self) -> Result<Vec<Role>> {
        crud::get_all::<role::Entity, Role>(&self.db).await
    }
}
