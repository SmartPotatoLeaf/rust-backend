use crate::adapters::web::models::company::{
    CompanyResponse, CreateCompanyRequest, SimplifiedCompanyResponse, UpdateCompanyRequest,
};
use spl_application::dtos::company::{CreateCompanyDto, UpdateCompanyDto};
use spl_domain::entities::company::Company;
use spl_shared::{map_mirror, maps_to};

map_mirror!(CreateCompanyRequest, CreateCompanyDto { name, description });

map_mirror!(UpdateCompanyRequest, UpdateCompanyDto { name, description });

map_mirror!(
    CompanyResponse,
    Company {
        id,
        name,
        description,
        created_at,
        updated_at,
    }
);

maps_to!(SimplifiedCompanyResponse {
    id,
    name,
} #from [Company]);
