use crate::adapters::web::models::user::SimplifiedUserResponse;
use crate::adapters::web::models::{
    auth::{LoginRequest, RegisterRequest},
    user::{
        ChangePasswordRequest, FullUserResponse, RoleResponse, SimplifiedRoleResponse,
        UpdateProfileRequest, UpdateUserRequest, UserResponse,
    },
};
use spl_application::dtos::user::{
    ChangePasswordDto, CreateUserDto, LoginDto, UpdateProfileDto, UpdateUserDto,
};
use spl_domain::entities::user::{Role, User};
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
        name,
        surname,
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
        name,
        surname,
        role,
        company_id,
    }
);

map_mirror!(
    UpdateProfileRequest,
    UpdateProfileDto {
        email,
        name,
        surname,
    }
);

map_mirror!(
    ChangePasswordRequest,
    ChangePasswordDto {
        current_password,
        new_password,
    }
);

impl From<User> for SimplifiedUserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            name: value.name,
            surname: value.surname,
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
            name: user.name,
            surname: user.surname,
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
            name: user.name,
            surname: user.surname,
            role: user.role.name,
            company: user.company.map(|c| c.into()),
            created_at: user.created_at,
        }
    }
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        Self {
            id: role.id,
            name: role.name,
            level: role.level,
        }
    }
}

impl From<Role> for SimplifiedRoleResponse {
    fn from(role: Role) -> Self {
        Self {
            id: role.id,
            name: role.name,
        }
    }
}
