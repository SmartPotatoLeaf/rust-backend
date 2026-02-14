use crate::adapters::persistence::entities::user::user::{ActiveModel, Model};
use sea_orm::Set;
use spl_domain::entities::company::Company;
use spl_domain::entities::user::{Role, User};
use spl_shared::error::AppError;
use spl_shared::traits::IntoWithContext;

pub struct UserMapperContext {
    pub role: Role,
    pub company: Option<Company>,
}

impl IntoWithContext<User, UserMapperContext> for Model {
    type Error = AppError;

    fn into_with_context(self, context: UserMapperContext) -> Result<User, Self::Error> {
        Ok(User {
            id: self.id,
            username: self.username,
            email: self.email,
            password_hash: self.password_hash,
            name: self.name,
            surname: self.surname,
            role: context.role,
            company: context.company,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
        })
    }
}

impl From<User> for ActiveModel {
    fn from(entity: User) -> Self {
        Self {
            id: Set(entity.id),
            username: Set(entity.username),
            email: Set(entity.email),
            password_hash: Set(entity.password_hash),
            name: Set(entity.name),
            surname: Set(entity.surname),
            role_id: Set(entity.role.id),
            company_id: Set(entity.company.map(|c| c.id)), // Extract ID from nested company
            created_at: Set(entity.created_at.into()),
            updated_at: Set(entity.updated_at.into()),
        }
    }
}
