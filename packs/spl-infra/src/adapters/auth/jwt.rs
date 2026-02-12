use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use spl_domain::ports::auth::TokenGenerator;
use spl_shared::config::AppConfig;
use spl_shared::error::{AppError, Result};
use std::sync::Arc;

pub struct JwtTokenGenerator {
    config: Arc<AppConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    role: String,
    #[serde(flatten)]
    extra: serde_json::Value,
}

impl JwtTokenGenerator {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

impl TokenGenerator for JwtTokenGenerator {
    fn generate(&self, sub: &str, claims: serde_json::Value) -> Result<String> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(
                self.config.server.jwt_expiration_hours as i64,
            ))
            .expect("valid timestamp")
            .timestamp() as usize;

        let role = claims
            .get("role")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::AuthError("Missing role in claims".to_string()))?
            .to_string();

        let mut extra = claims;
        if let Some(obj) = extra.as_object_mut() {
            obj.remove("role");
        }

        let claims = Claims {
            sub: sub.to_owned(),
            exp: expiration,
            role,
            extra,
        };

        // Use secret from config
        let secret = &self.config.server.jwt_secret;

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| AppError::AuthError(format!("Failed to generate token for {sub}: {e}")))
    }

    fn validate(&self, token: &str) -> Result<serde_json::Value> {
        let secret = &self.config.server.jwt_secret;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| AppError::AuthError(format!("Failed to validate token: {e}")))?;

        let value = serde_json::to_value(token_data.claims).unwrap_or_default();
        // Ensure role is kept in the returned value if needed, or structured differently.
        // The flatten behavior puts extra fields at top level. 'role' is a struct field.
        // serde_json::to_value(Claims { ... }) will produce { "sub":..., "exp":..., "role":..., ...extra }
        // So it works as expected.
        Ok(value)
    }
}
