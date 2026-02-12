use async_trait::async_trait;
use spl_shared::error::Result;

#[async_trait]
pub trait PasswordEncoder: Send + Sync {
    fn hash(&self, password: &str) -> Result<String>;
    fn verify(&self, password: &str, hash: &str) -> Result<bool>;
}

#[async_trait]
pub trait TokenGenerator: Send + Sync {
    fn generate(&self, sub: &str, claims: serde_json::Value) -> Result<String>;
    fn validate(&self, token: &str) -> Result<serde_json::Value>;
}
