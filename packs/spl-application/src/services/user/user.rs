use crate::dtos::user::{CreateUserDto, UpdateUserDto};
use crate::mappers::user::{UserCreationContext, UserUpdateContext};
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

    /// Validates if the requester can perform actions on the target user
    /// Returns true if allowed, false otherwise
    async fn can_manage_user(&self, requester: &User, target_user: &User) -> Result<bool> {
        let requester_level = requester.role.level;

        // Admin can manage anyone
        if requester_level >= 100 {
            return Ok(true);
        }

        // Supervisor can only manage users in their own company
        if requester_level >= 50 {
            let requester_company_id = requester.company.as_ref().map(|c| c.id);
            let target_company_id = target_user.company.as_ref().map(|c| c.id);

            // Both must have a company and they must match
            if requester_company_id.is_some()
                && requester_company_id == target_company_id
                && target_user.role.level < requester_level
            {
                return Ok(true);
            }
        }

        // Users cannot manage other users
        Ok(false)
    }

    pub async fn create_user(&self, creator: &User, dto: CreateUserDto) -> Result<User> {
        // Fetch creator's role to check permissions
        let creator_role = &creator.role;

        let target_role_id;
        let target_company_id;

        // Level 100 = Admin
        // Level 50 = Supervisor
        // Level 10 = User

        if creator_role.level >= 100 {
            // Admin Logic
            let target_role_name = dto.role.as_ref().ok_or_else(|| {
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

            let target_role = if let Some(r_name) = &dto.role {
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

        // get target role
        let target_role = self
            .role_repo
            .get_by_id(target_role_id)
            .await?
            .ok_or_else(|| AppError::ValidationError("Role lost".to_string()))?;

        if target_role.level >= 50 && dto.email.is_none() {
            return Err(AppError::ValidationError(
                "Email is required for Supervisor and Admin roles".to_string(),
            ));
        }

        let password_hash = self.password_encoder.hash(&dto.password)?;

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

    pub async fn update_user(
        &self,
        requester: &User,
        target_id: Uuid,
        dto: UpdateUserDto,
    ) -> Result<User> {
        // Get the target user
        let target_user = self
            .user_repo
            .get_by_id(target_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check permissions
        if !self.can_manage_user(requester, &target_user).await? {
            return Err(AppError::Forbidden);
        }

        let requester_level = requester.role.level;

        // Determine new role if role_name is provided
        let new_role = if let Some(ref role_name) = dto.role {
            let role = self
                .role_repo
                .get_by_name(role_name)
                .await?
                .ok_or_else(|| AppError::ValidationError("Invalid role name".to_string()))?;

            // Admins can assign any role
            // Supervisors can only assign roles lower than their own
            if requester_level < 100 && role.level >= requester_level {
                return Err(AppError::Forbidden);
            }

            Some(role)
        } else {
            None
        };

        let role = new_role.clone().unwrap_or(target_user.role.clone());

        if role.level >= 50 && dto.email.is_none() && target_user.email.is_none() {
            return Err(AppError::ValidationError(
                "Email is required for Supervisor and Admin roles".to_string(),
            ));
        }

        // Determine new company if company_id is provided
        let new_company = if let Some(cid) = dto.company_id {
            if requester_level < 100 {
                // Only admins can change company
                return Err(AppError::Forbidden);
            }
            self.company_repo.get_by_id(cid).await?
        } else {
            None
        };

        // Hash new password if provided
        let password_hash = if let Some(ref password) = dto.password {
            Some(self.password_encoder.hash(password)?)
        } else {
            None
        };

        // Check username uniqueness if changed
        if let Some(ref new_username) = dto.username {
            if new_username != &target_user.username {
                let target_company_id = new_company
                    .as_ref()
                    .map(|c| c.id)
                    .or(target_user.company.as_ref().map(|c| c.id));

                let exists = self
                    .user_repo
                    .get_by_username_and_company(new_username, target_company_id)
                    .await?;

                if exists.is_some() {
                    return Err(AppError::UserAlreadyExists);
                }
            }
        }

        let context = UserUpdateContext {
            current_user: target_user,
            password_hash,
            role: new_role,
            company: new_company,
        };

        let updated_user = dto.into_with_context(context)?;

        self.user_repo.update(updated_user).await
    }

    pub async fn delete_user(&self, requester: &User, target_id: Uuid) -> Result<User> {
        // Get the target user
        let target_user = self
            .user_repo
            .get_by_id(target_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check permissions
        if !self.can_manage_user(requester, &target_user).await? {
            return Err(AppError::Forbidden);
        }

        // Prevent self-deletion
        if requester.id == target_id {
            return Err(AppError::ValidationError(
                "Cannot delete your own account".to_string(),
            ));
        }

        self.user_repo.delete(target_id).await
    }
}
