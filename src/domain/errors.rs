use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

impl DomainError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn external(msg: impl Into<String>) -> Self {
        Self::ExternalService(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, DomainError>;
