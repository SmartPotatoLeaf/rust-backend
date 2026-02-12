use crate::dtos::user::LoginDto;
use spl_domain::ports::auth::{PasswordEncoder, TokenGenerator};
use spl_domain::ports::repositories::user::UserRepository;
use spl_shared::error::{AppError, Result};
use std::sync::Arc;

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

    pub async fn login(&self, dto: LoginDto) -> Result<String> {
        // Validate that at least one of username or email is provided
        if dto.username.is_none() && dto.email.is_none() {
            return Err(AppError::ValidationError(
                "Either username or email must be provided".to_string(),
            ));
        }

        let user = self
            .user_repo
            .get_by_username_or_email_and_company(
                dto.username,
                dto.email,
                dto.company_id,
            )
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !self
            .password_encoder
            .verify(&dto.password, &user.password_hash)?
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
