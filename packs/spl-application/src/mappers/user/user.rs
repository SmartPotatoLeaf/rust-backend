use crate::dtos::user::{CreateUserDto, UpdateProfileDto, UpdateUserDto};
use chrono::Utc;
use spl_domain::entities::company::Company;
use spl_domain::entities::user::{Role, User};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;

pub struct UserCreationContext {
    pub password_hash: String,
    pub role: Role,
    pub company: Option<Company>,
}

impl IntoWithContext<User, UserCreationContext> for CreateUserDto {
    type Error = AppError;

    fn into_with_context(self, context: UserCreationContext) -> Result<User> {
        Ok(User {
            id: Uuid::new_v4(),
            username: self.username,
            email: self.email,
            password_hash: context.password_hash,
            name: self.name,
            surname: self.surname,
            role: context.role,
            company: context.company,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

impl IntoWithContext<User, User> for UpdateProfileDto {
    type Error = AppError;

    fn into_with_context(self, current: User) -> Result<User> {
        Ok(User {
            id: current.id,
            username: current.username,
            email: self.email.or(current.email),
            password_hash: current.password_hash,
            name: self.name.or(current.name),
            surname: self.surname.or(current.surname),
            role: current.role,
            company: current.company,
            created_at: current.created_at,
            updated_at: Utc::now(),
        })
    }
}
pub struct UserUpdateContext {
    pub current_user: User,
    pub password_hash: Option<String>,
    pub role: Option<Role>,
    pub company: Option<Company>,
}

impl IntoWithContext<User, UserUpdateContext> for UpdateUserDto {
    type Error = AppError;

    fn into_with_context(self, context: UserUpdateContext) -> Result<User> {
        let current = context.current_user;

        Ok(User {
            id: current.id,
            username: self.username.unwrap_or(current.username),
            email: self.email.or(current.email),
            password_hash: context.password_hash.unwrap_or(current.password_hash),
            name: self.name.or(current.name),
            surname: self.surname.or(current.surname),
            role: context.role.unwrap_or(current.role),
            company: if self.company_id.is_some() {
                context.company
            } else {
                current.company
            },
            created_at: current.created_at,
            updated_at: Utc::now(),
        })
    }
}
