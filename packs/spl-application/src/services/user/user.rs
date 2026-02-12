use crate::dtos::user::CreateUserDto;
use crate::mappers::user::UserCreationContext;
use spl_domain::entities::user::User;
use spl_domain::ports::auth::PasswordEncoder;
use spl_domain::ports::repositories::company::CompanyRepository;
use spl_domain::ports::repositories::user::{RoleRepository, UserRepository};
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    role_repo: Arc<dyn RoleRepository>,
    company_repo: Arc<dyn CompanyRepository>,
    password_encoder: Arc<dyn PasswordEncoder>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        role_repo: Arc<dyn RoleRepository>,
        company_repo: Arc<dyn CompanyRepository>,
        password_encoder: Arc<dyn PasswordEncoder>,
    ) -> Self {
        Self {
            user_repo,
            role_repo,
            company_repo,
            password_encoder,
        }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<User>> {
        self.user_repo.get_by_id(id).await
    }

    pub async fn create_user(&self, creator: &User, dto: CreateUserDto) -> Result<User> {
        // Fetch creator's role to check permissions
        let creator_role = &creator.role; // Should not happen if data integrity is good

        let target_role_id;
        let target_company_id;

        // Level 100 = Admin
        // Level 50 = Supervisor
        // Level 10 = User

        if creator_role.level >= 100 {
            // Admin Logic
            let target_role_name = dto.role_name.as_ref().ok_or_else(|| {
                AppError::ValidationError("Role name required for Admin creation".to_string())
            })?;

            let target_role = self
                .role_repo
                .get_by_name(target_role_name)
                .await?
                .ok_or_else(|| AppError::ValidationError("Invalid target role name".to_string()))?;

            target_role_id = target_role.id;

            if target_role.level >= 100 {
                // Creating another Admin
                target_company_id = None;
            } else {
                // Creating Supervisor or User
                target_company_id = Some(dto.company_id.ok_or_else(|| {
                    AppError::ValidationError("Company ID required for this role".to_string())
                })?);
            }
        } else if creator_role.level >= 50 {
            // Supervisor Logic
            // Can only create users (level < 50)

            let target_role = if let Some(r_name) = &dto.role_name {
                self.role_repo.get_by_name(r_name).await?.ok_or_else(|| {
                    AppError::ValidationError("Invalid target role name".to_string())
                })?
            } else {
                self.role_repo.get_by_name("user").await?.ok_or_else(|| {
                    AppError::ValidationError("Default 'User' role not found".to_string())
                })?
            };

            if target_role.level >= creator_role.level {
                return Err(AppError::Forbidden);
            }

            target_role_id = target_role.id;
            target_company_id = creator.company.as_ref().map(|c| c.id); // Inherit company_id from creator
        } else {
            // Users cannot create users
            return Err(AppError::Forbidden);
        }

        // Check uniqueness match
        let exists = self
            .user_repo
            .get_by_username_and_company(&dto.username, target_company_id)
            .await?;
        if exists.is_some() {
            return Err(AppError::UserAlreadyExists);
        }

        let password_hash = self.password_encoder.hash(&dto.password)?;

        // get target role
        let target_role = self
            .role_repo
            .get_by_id(target_role_id)
            .await?
            .ok_or_else(|| AppError::ValidationError("Role lost".to_string()))?;

        // get target company if needed
        let target_company = if let Some(cid) = target_company_id {
            self.company_repo.get_by_id(cid).await?
        } else {
            None
        };

        if target_company_id.is_some() && target_company.is_none() {
            return Err(AppError::ValidationError(
                "Target company not found".to_string(),
            ));
        }

        let context = UserCreationContext {
            password_hash,
            role: target_role,
            company: target_company,
        };

        let new_user = dto.into_with_context(context)?;

        self.user_repo.create(new_user).await
    }
}
