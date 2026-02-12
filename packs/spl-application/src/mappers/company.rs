use crate::dtos::company::{CreateCompanyDto, UpdateCompanyDto};
use chrono::Utc;
use spl_domain::entities::company::Company;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use uuid::Uuid;

impl From<CreateCompanyDto> for Company {
    fn from(dto: CreateCompanyDto) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: dto.name,
            description: dto.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl IntoWithContext<Company, Company> for UpdateCompanyDto {
    type Error = AppError;

    fn into_with_context(self, context: Company) -> Result<Company> {
        Ok(Company {
            name: self.name.unwrap_or(context.name),
            description: self.description.or(context.description),
            updated_at: Utc::now(),
            ..context
        })
    }
}
