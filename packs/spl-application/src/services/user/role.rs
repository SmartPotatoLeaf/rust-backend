use spl_domain::entities::user::Role;
use spl_domain::ports::repositories::user::RoleRepository;
use spl_shared::error::Result;
use std::sync::Arc;

pub struct RoleService {
    repo: Arc<dyn RoleRepository>,
}

impl RoleService {
    pub fn new(repo: Arc<dyn RoleRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<Role>> {
        self.repo.get_by_id(id).await
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Role>> {
        self.repo.get_by_name(name).await
    }
}
