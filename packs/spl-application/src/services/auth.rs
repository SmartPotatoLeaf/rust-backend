use spl_domain::ports::auth::{PasswordEncoder, TokenGenerator};
use spl_domain::ports::repositories::user::UserRepository;
use spl_shared::error::{AppError, Result};
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    password_encoder: Arc<dyn PasswordEncoder>,
    token_generator: Arc<dyn TokenGenerator>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        password_encoder: Arc<dyn PasswordEncoder>,
        token_generator: Arc<dyn TokenGenerator>,
    ) -> Self {
        Self {
            user_repo,
            password_encoder,
            token_generator,
        }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        company_id: Option<Uuid>,
    ) -> Result<String> {
        let user = self
            .user_repo
            .get_by_username_and_company(username, company_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !self
            .password_encoder
            .verify(password, &user.password_hash)?
        {
            return Err(AppError::InvalidCredentials);
        }

        // Payload with role
        let claims = serde_json::json!({
            "role": &user.role.name
        });

        self.token_generator.generate(&user.id.to_string(), claims)
    }

    pub fn validate_token(&self, token: &str) -> Result<serde_json::Value> {
        self.token_generator.validate(token)
    }
}
