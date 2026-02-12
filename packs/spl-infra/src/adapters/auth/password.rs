use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use spl_domain::ports::auth::PasswordEncoder;
use spl_shared::error::{AppError, Result};

pub struct Argon2PasswordEncoder;

impl Argon2PasswordEncoder {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordEncoder for Argon2PasswordEncoder {
    fn hash(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::AuthError(format!("Failed to hash password: {e}")))?
            .to_string();
        Ok(password_hash)
    }

    fn verify(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::AuthError(format!("Failed to parse hash: {e}")))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
