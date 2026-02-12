use async_trait::async_trait;
use spl_shared::error::Result;

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn upload(&self, path: &str, content: Vec<u8>) -> Result<()>;
    async fn delete(&self, path: &str) -> Result<()>;
}
