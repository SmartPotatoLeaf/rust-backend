use crate::adapters::web::models::company::{
    CompanyResponse, CreateCompanyRequest, SimplifiedCompanyResponse, UpdateCompanyRequest,
};
use spl_application::dtos::company::{CreateCompanyDto, UpdateCompanyDto};
use spl_domain::entities::company::Company;

impl From<CreateCompanyRequest> for CreateCompanyDto {
    fn from(req: CreateCompanyRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<UpdateCompanyRequest> for UpdateCompanyDto {
    fn from(req: UpdateCompanyRequest) -> Self {
        Self {
            name: req.name,
            description: req.description,
        }
    }
}

impl From<Company> for CompanyResponse {
    fn from(company: Company) -> Self {
        Self {
            id: company.id,
            name: company.name,
            description: company.description,
            created_at: company.created_at,
            updated_at: company.updated_at,
        }
    }
}

impl From<Company> for SimplifiedCompanyResponse {
    fn from(company: Company) -> Self {
        Self {
            id: company.id,
            name: company.name,
        }
    }
}
