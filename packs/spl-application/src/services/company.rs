use crate::dtos::company::{CreateCompanyDto, UpdateCompanyDto};

use spl_domain::entities::company::Company;
use spl_domain::entities::user::User;
use spl_domain::ports::repositories::company::CompanyRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::sync::Arc;
use uuid::Uuid;

pub struct CompanyService {
    company_repo: Arc<dyn CompanyRepository>,
}

impl CompanyService {
    pub fn new(company_repo: Arc<dyn CompanyRepository>) -> Self {
        Self { company_repo }
    }

    pub async fn create(&self, creator: &User, dto: CreateCompanyDto) -> Result<Company> {
        let role_level = creator.role.level;
        if role_level < 100 {
            return Err(AppError::Forbidden);
        }

        self.company_repo.create(dto.into()).await
    }

    pub async fn get_by_id(&self, requester: &User, id: Uuid) -> Result<Option<Company>> {
        let role_level = requester.role.level;

        if role_level < 100 {
            let user_company_id = requester.company.as_ref().map(|c| c.id);
            match user_company_id {
                Some(cid) if cid == id => {
                    // Allowed
                }
                _ => return Err(AppError::Forbidden),
            }
        }

        self.company_repo.get_by_id(id).await
    }

    pub async fn update(
        &self,
        requester: &User,
        id: Uuid,
        dto: UpdateCompanyDto,
    ) -> Result<Company> {
        let role_level = requester.role.level;

        if role_level >= 100 {
            // Admin allowed
        } else if role_level >= 50 {
            // Supervisor check
            let user_company_id = requester.company.as_ref().map(|c| c.id);
            if user_company_id != Some(id) {
                return Err(AppError::Forbidden);
            }
        } else {
            // User forbidden
            return Err(AppError::Forbidden);
        }

        let current = self
            .company_repo
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Company not found".to_string()))?;

        let updated = dto.into_with_context(current)?;

        self.company_repo.update(updated).await
    }

    pub async fn delete(&self, requester: &User, id: Uuid) -> Result<Company> {
        if requester.role.level < 100 {
            return Err(AppError::Forbidden);
        }
        self.company_repo.delete(id).await
    }
}
