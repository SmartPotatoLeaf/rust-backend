use spl_domain::entities::user::User;
use spl_domain::ports::repositories::company::CompanyRepository;
use spl_domain::ports::repositories::user::UserRepository;
use spl_shared::error::{AppError, Result};
use std::sync::Arc;
use uuid::Uuid;

pub struct AccessControlService {
    company_repo: Arc<dyn CompanyRepository>,
    user_repo: Arc<dyn UserRepository>,
}

impl AccessControlService {
    pub fn new(
        company_repo: Arc<dyn CompanyRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            company_repo,
            user_repo,
        }
    }

    /// Validates if a user has access to a specific company context.
    /// Returns the resolved Company ID to be used in queries.
    pub async fn validate_company_access(
        &self,
        requester: &User,
        requested_company_id: Option<Uuid>,
    ) -> Result<Uuid> {
        if requester.role.level >= 100 {
            // Admin: Must provide company_id to target a specific company
            let cid = requested_company_id.ok_or_else(|| {
                AppError::ValidationError(
                    "Company ID is required for Admin operations on company resources".to_string(),
                )
            })?;

            // Verify Company Exists
            if self.company_repo.get_by_id(cid).await?.is_none() {
                return Err(AppError::ValidationError(
                    "Target company not found".to_string(),
                ));
            }
            Ok(cid)
        } else {
            // Non-Admin: Must belong to a company
            let user_company_id = requester.company.as_ref().map(|c| c.id).ok_or_else(|| {
                AppError::ValidationError("User must belong to a company".to_string())
            })?;

            // If they requested a specific company, it MUST match their own
            if let Some(requested_cid) = requested_company_id {
                if requested_cid != user_company_id {
                    return Err(AppError::Forbidden);
                }
            }

            Ok(user_company_id)
        }
    }

    /// Returns a list of User IDs that the requester is allowed to access/filter by.
    /// - Admin: Can access anyone in the target company.
    /// - Supervisor: Can access anyone in their own company.
    /// - User: Can only access themselves.
    pub async fn get_accessible_user_ids(
        &self,
        requester: &User,
        target_company_id: Option<Uuid>,
    ) -> Result<Vec<Uuid>> {
        let mut target_user_ids = Vec::new();

        if requester.role.level >= 100 {
            // Admin
            if let Some(company_id) = target_company_id {
                // Fetch all users of that company
                let users: Vec<Uuid> = self
                    .user_repo
                    .get_by_company_id(company_id)
                    .await?
                    .iter()
                    .map(|u| u.id)
                    .collect();
                target_user_ids.extend(users);
            } else {
                // For simplified compatibility, default to self
                target_user_ids.push(requester.id);
            }
        } else if requester.role.level >= 50 {
            // Supervisor
            if let Some(company) = &requester.company {
                let users = self.user_repo.get_by_company_id(company.id).await?;
                for user in users {
                    target_user_ids.push(user.id);
                }
            } else {
                // Supervisor without company (shouldn't happen but fallback)
                target_user_ids.push(requester.id);
            }
        } else {
            // Regular user
            target_user_ids.push(requester.id);
        }

        Ok(target_user_ids)
    }
}
