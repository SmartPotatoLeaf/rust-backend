pub use sea_orm_migration::prelude::*;

mod m20260130_000001_create_role_table;
mod m20260130_000002_create_user_table;
mod m20260130_000003_create_company_table;
mod m20260205_000004_create_recommendation_tables;
mod m20260206_000005_create_diagnostics_tables;
mod m20260206_000006_create_feedback_tables;
mod m20260207_000007_create_plots_table;
mod m20260207_000008_create_images_table;
mod m20260209_000009_seed_recommendations;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260130_000001_create_role_table::Migration),
            Box::new(m20260130_000002_create_user_table::Migration),
            Box::new(m20260130_000003_create_company_table::Migration),
            Box::new(m20260205_000004_create_recommendation_tables::Migration),
            Box::new(m20260206_000005_create_diagnostics_tables::Migration),
            Box::new(m20260206_000006_create_feedback_tables::Migration),
            Box::new(m20260207_000007_create_plots_table::Migration),
            Box::new(m20260207_000008_create_images_table::Migration),
            Box::new(m20260209_000009_seed_recommendations::Migration),
        ]
    }
}
