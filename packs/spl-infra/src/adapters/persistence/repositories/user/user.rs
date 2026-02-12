use crate::adapters::persistence::entities;
use crate::adapters::persistence::entities::company;
use crate::adapters::persistence::entities::user::{role, user};
use crate::adapters::persistence::mappers::user::user::UserMapperContext;
use sea_orm::*;
use spl_domain::entities::company::Company;
use spl_domain::entities::user::{Role, User};
use spl_domain::ports::repositories::company::CompanyRepository;
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::user::{RoleRepository, UserRepository};
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

pub struct DbUserRepository {
    db: DatabaseConnection,
    role_repository: Arc<dyn RoleRepository>,
    company_repository: Arc<dyn CompanyRepository>,
}

impl DbUserRepository {
    pub fn new(
        db: DatabaseConnection,
        role_repository: Arc<dyn RoleRepository>,
        company_repository: Arc<dyn CompanyRepository>,
    ) -> Self {
        Self {
            db,
            role_repository,
            company_repository,
        }
    }

    async fn find_relations(
        &self,
        role_id: i32,
        company_id: &Option<Uuid>,
    ) -> Result<(Role, Option<Company>)> {
        let company_future = async {
            match company_id {
                Some(cid) => self.company_repository.get_by_id(cid.clone()).await,
                None => Ok(None),
            }
        };

        let (role_opt, company_opt) =
            tokio::try_join!(self.role_repository.get_by_id(role_id), company_future)?;

        let target_role =
            role_opt.ok_or(AppError::NotFound(format!("No role with id {}", role_id)))?;

        Ok((target_role, company_opt))
    }

    async fn validate_relations(&self, user: &User) -> Result<(Role, Option<Company>)> {
        let (role, company) = self
            .find_relations(user.role.id, &user.company.as_ref().map(|c| c.id))
            .await?;

        Ok((role, company))
    }

    fn map_related_model(
        &self,
        result: Option<(user::Model, Option<role::Model>, Option<company::Model>)>,
    ) -> Result<Option<User>> {
        match result {
            Some((user_model, Some(role_model), company_model)) => {
                let role: Role = role_model.into();
                let company: Option<Company> = company_model.map(|c| c.into());

                let context = UserMapperContext { role, company };

                let user = user_model.into_with_context(context)?;
                Ok(Some(user))
            }
            Some((user_model, None, _)) => Err(AppError::NotFound(format!(
                "User {} has no role assigned (integrity error)",
                user_model.id
            ))),
            None => Ok(None),
        }
    }

    async fn with_map<F>(&self, action: F) -> Result<User>
    where
        F: AsyncFnOnce() -> Result<(user::Model, Role, Option<Company>)>,
    {
        let (user, role, company) = action().await?;

        let context = UserMapperContext { role, company };
        let user = user.into_with_context(context)?;
        Ok(user)
    }
}

#[async_trait::async_trait]
impl CrudRepository<User, Uuid> for DbUserRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let result = user::Entity::find_by_id(id)
            .find_also_related(role::Entity)
            .find_also_related(entities::company::Entity)
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        self.map_related_model(result)
    }

    async fn create(&self, user: User) -> Result<User> {
        self.with_map(|| async {
            let (target_role, target_company) = self.validate_relations(&user).await?;

            let result = crud::create_model::<user::Entity, User>(&self.db, user).await?;

            Ok((result, target_role, target_company))
        })
        .await
    }

    async fn update(&self, user: User) -> Result<User> {
        self.with_map(|| async {
            let (target_role, target_company) = self.validate_relations(&user).await?;

            let result = crud::update_model::<user::Entity, User>(&self.db, user).await?;

            Ok((result, target_role, target_company))
        })
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<User> {
        self.with_map(|| async {
            let result = crud::delete_model::<user::Entity, User, Uuid>(&self.db, id).await?;

            let (target_role, target_company) = self
                .find_relations(result.role_id, &result.company_id)
                .await?;

            Ok((result, target_role, target_company))
        })
        .await
    }
}
#[async_trait::async_trait]
impl UserRepository for DbUserRepository {
    async fn get_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<User>> {
        let models = user::Entity::find()
            .filter(user::Column::Id.is_in(ids))
            .find_also_related(role::Entity)
            .find_also_related(company::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        models
            .into_iter()
            .map(|model| {
                self.map_related_model(Some((model.0, model.1.into(), model.2.into())))
                    .and_then(|o| {
                        o.ok_or_else(|| {
                            AppError::NotFound("Related role not found for user".to_string())
                        })
                    })
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn get_by_username_and_company(
        &self,
        username: &str,
        company_id: Option<Uuid>,
    ) -> Result<Option<User>> {
        let mut query = user::Entity::find()
            .filter(user::Column::Username.eq(username))
            .find_also_related(role::Entity)
            .find_also_related(entities::company::Entity);

        if let Some(cid) = company_id {
            query = query.filter(user::Column::CompanyId.eq(cid));
        } else {
            // If company_id is None, filter for users with no company (global scope/admin)
            query = query.filter(user::Column::CompanyId.is_null());
        }

        let result = query.one(&self.db).await.map_err(AppError::from)?;

        self.map_related_model(result)
    }

    async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<User>> {
        let models = user::Entity::find()
            .filter(user::Column::CompanyId.eq(company_id))
            .find_also_related(role::Entity)
            .find_also_related(entities::company::Entity)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        let mut users = Vec::with_capacity(models.len());
        for model in models {
            if let Some(user) = self.map_related_model(Some(model))? {
                users.push(user);
            }
        }
        Ok(users)
    }
}
