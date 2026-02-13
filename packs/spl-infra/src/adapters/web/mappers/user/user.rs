use crate::adapters::web::models::user::SimplifiedUserResponse;
use crate::adapters::web::models::{
    auth::{LoginRequest, RegisterRequest},
    user::{FullUserResponse, UpdateUserRequest, UserResponse},
};
use spl_application::dtos::user::{CreateUserDto, LoginDto, UpdateUserDto};
use spl_domain::entities::user::User;
use spl_shared::map_mirror;

map_mirror!(
    LoginRequest,
    LoginDto {
        username,
        email,
        password,
        company_id,
    }
);

map_mirror!(
    RegisterRequest,
    CreateUserDto {
        username,
        email,
        password,
        company_id,
        role,
    }
);

map_mirror!(
    UpdateUserRequest,
    UpdateUserDto {
        username,
        email,
        password,
        role,
        company_id,
    }
);

impl From<User> for SimplifiedUserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            role: value.role.name,
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
            created_at: user.created_at,
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
            created_at: user.created_at,
        }
    }
}
