pub mod company;
pub mod diagnostics;
pub mod image;
pub mod plot;
pub mod recommendation;
pub mod user;

pub use company::DbCompanyRepository;
pub use diagnostics::{DbLabelRepository, DbMarkTypeRepository, DbPredictionRepository};
// feedback repo is likely WIP or different structure, checking...
pub use image::DbImageRepository;
pub use plot::DbPlotRepository;
pub use recommendation::{DbCategoryRepository, DbRecommendationRepository};
pub use user::{DbRoleRepository, DbUserRepository};
