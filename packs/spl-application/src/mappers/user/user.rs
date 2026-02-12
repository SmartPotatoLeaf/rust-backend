use crate::dtos::user::CreateUserDto;
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
            role: context.role,
            company: context.company,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
