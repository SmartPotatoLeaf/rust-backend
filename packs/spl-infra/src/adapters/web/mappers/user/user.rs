use crate::adapters::web::models::{
    auth::{LoginRequest, RegisterRequest},
    user::{FullUserResponse, UpdateUserRequest, UserResponse},
};
use spl_application::dtos::user::{CreateUserDto, LoginDto, UpdateUserDto};
use spl_domain::entities::user::User;

impl From<LoginRequest> for LoginDto {
    fn from(req: LoginRequest) -> Self {
        Self {
            username: req.username,
            email: req.email,
            password: req.password,
            company_id: req.company_id,
        }
    }
}

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

impl From<UpdateUserRequest> for UpdateUserDto {
    fn from(req: UpdateUserRequest) -> Self {
        Self {
            username: req.username,
            email: req.email,
            password: req.password,
            role_name: req.role,
            company_id: req.company_id,
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
