use crate::adapters::web::models::{
    auth::RegisterRequest,
    user::{FullUserResponse, UserResponse},
};
use spl_application::dtos::user::CreateUserDto;
use spl_domain::entities::user::User;

impl From<RegisterRequest> for CreateUserDto {
    fn from(req: RegisterRequest) -> Self {
        Self {
            username: req.username,
            email: req.email,
            password: req.password,
            company_id: req.company_id,
            role_name: req.role,
        }
    }
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role.name,
            company_id: user.company.map(|c| c.id),
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

impl From<User> for FullUserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role.name,
            company: user.company.map(|c| c.into()),
            created_at: user.created_at.to_rfc3339(),
        }
    }
}
