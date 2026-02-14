use anyhow::Result;
use spl_domain::entities::user::User;
use spl_domain::ports::auth::PasswordEncoder;
use spl_domain::ports::repositories::user::{RoleRepository, UserRepository};
use spl_shared::config::AdminConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub async fn seed_admin_user(
    admin_config: &AdminConfig,
    role_repo: &Arc<dyn RoleRepository>,
    user_repo: &Arc<dyn UserRepository>,
    password_encoder: &Arc<dyn PasswordEncoder>,
) -> Result<()> {
    info!("Checking for Admin user seeding...");

    // Get admin role or fail early
    let admin_role = match role_repo.get_by_name("admin").await {
        Ok(Some(role)) => role,
        Ok(None) => {
            error!("'Admin' role not found. Cannot seed admin user.");
            return Ok(());
        }
        Err(e) => {
            error!("Failed to fetch 'Admin' role: {}", e);
            return Ok(());
        }
    };

    // Check if admin user already exists
    let existing_user = match user_repo
        .get_by_username_and_company(&admin_config.username, None)
        .await
    {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to check for existing admin user: {}", e);
            return Ok(());
        }
    };

    if existing_user.is_some() {
        info!("Admin user already exists. Skipping.");
        return Ok(());
    }

    // Create new admin user
    info!("Admin user not found. Creating...");

    let password_hash = match password_encoder.hash(&admin_config.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash admin password: {}", e);
            return Ok(());
        }
    };

    let new_admin = User {
        id: Uuid::new_v4(),
        username: admin_config.username.clone(),
        email: Some(admin_config.email.clone()),
        password_hash,
        name: None,
        surname: None,
        role: admin_role,
        company: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match user_repo.create(new_admin).await {
        Ok(_) => info!("Admin user seeded successfully."),
        Err(e) => error!("Failed to seed admin user: {}", e),
    }

    Ok(())
}

pub async fn load_role_cache(role_repo: &Arc<dyn RoleRepository>) -> Result<HashMap<String, i16>> {
    info!("Loading role cache...");

    let cache = role_repo
        .get_all()
        .await?
        .into_iter()
        .map(|r| (r.name, r.level))
        .collect::<HashMap<String, i16>>();

    Ok(cache)
}
