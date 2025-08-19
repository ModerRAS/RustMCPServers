use thiserror::Error;

/// 应用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Task already acquired")]
    TaskAlreadyAcquired,
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// 应用结果类型
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    pub fn task_not_found(id: impl Into<String>) -> Self {
        Self::TaskNotFound(id.into())
    }
    
    pub fn authorization(msg: impl Into<String>) -> Self {
        Self::Authorization(msg.into())
    }
    
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }
    
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }
    
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }
    
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::TaskNotFound(_) => "NOT_FOUND",
            AppError::TaskAlreadyAcquired => "CONFLICT",
            AppError::Authorization(_) => "AUTHORIZATION_ERROR",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }
    
    pub fn http_status(&self) -> u16 {
        match self {
            AppError::Validation(_) => 400,
            AppError::TaskNotFound(_) => 404,
            AppError::TaskAlreadyAcquired => 409,
            AppError::Authorization(_) => 401,
            AppError::RateLimitExceeded => 429,
            AppError::ServiceUnavailable(_) => 503,
            AppError::Configuration(_) => 500,
            AppError::Database(_) => 500,
            AppError::Internal(_) => 500,
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self::Internal(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        Self::Internal(s.to_string())
    }
}