use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("No content: {0}")]
    NoContent(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Forbidden")]
    Forbidden,

    #[error("User already exists inside this company")]
    UserAlreadyExists,

    #[error("Integration error ({integration}): {message}")]
    IntegrationError {
        integration: String,
        message: String,
    },

    #[error("Integration timeout: {0}")]
    IntegrationTimeout(String),

    #[error("Integration unavailable: {0}")]
    IntegrationUnavailable(String),

    #[error("Invalid credentials")]
    InvalidCredentials,
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
