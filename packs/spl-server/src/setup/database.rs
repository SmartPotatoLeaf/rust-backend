use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use spl_migration::Migrator;
use spl_shared::config::DatabaseConfig;
use std::time::Duration;
use tracing::info;

pub async fn initialize_database(config: &DatabaseConfig) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(config.url.clone());
    opt.max_connections(config.max_connections.unwrap_or(20))
        .min_connections(config.min_connections.unwrap_or(5))
        .connect_timeout(Duration::from_secs(config.connect_timeout.unwrap_or(10)))
        .idle_timeout(Duration::from_secs(config.idle_timeout.unwrap_or(300)))
        .max_lifetime(Duration::from_secs(config.max_lifetime.unwrap_or(1800)));

    let db = Database::connect(opt).await?;
    info!("Connected to database: {}", config.url);

    // Run migrations
    Migrator::up(&db, None).await?;
    info!("Migrations executed successfully.");

    Ok(db)
}
