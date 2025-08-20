//! 工具模块

pub mod logging;
pub mod validation;

/// 通用工具函数
pub mod utils {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    /// 生成唯一的请求ID
    pub fn generate_request_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// 获取当前时间戳（毫秒）
    pub fn current_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_millis() as u64
    }

    /// 获取当前时间戳（秒）
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs()
    }

    /// 格式化持续时间
    pub fn format_duration(duration: Duration) -> String {
        if duration.as_secs() > 0 {
            format!("{}.{:03}s", duration.as_secs(), duration.subsec_millis())
        } else {
            format!("{}ms", duration.as_millis())
        }
    }

    /// 计算字符串的哈希值
    pub fn hash_string(s: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 安全地解析JSON值
    pub fn safe_json_parse(json: &str) -> Option<serde_json::Value> {
        serde_json::from_str(json).ok()
    }

    /// 截断字符串
    pub fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }

    /// 检查字符串是否为有效的JSON
    pub fn is_valid_json(s: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(s).is_ok()
    }

    /// 人类可读的字节大小格式
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        
        if bytes == 0 {
            return "0 B".to_string();
        }
        
        let bytes = bytes as f64;
        let base = 1024_f64;
        let i = (bytes.ln() / base.ln()).floor() as usize;
        let unit = UNITS.get(i).unwrap_or(&UNITS[UNITS.len() - 1]);
        
        format!("{:.2} {}", bytes / base.powi(i as i32), unit)
    }

    /// 获取系统信息
    pub fn get_system_info() -> serde_json::Value {
        serde_json::json!({
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "target": std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()),
            "profile": std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()),
        })
    }
}

/// 错误处理工具
pub mod error {
    use thiserror::Error;

    /// 应用错误类型
    #[derive(Error, Debug)]
    pub enum AppError {
        #[error("Validation error: {0}")]
        Validation(String),
        
        #[error("JSON parsing error: {0}")]
        JsonParse(String),
        
        #[error("Schema error: {0}")]
        Schema(String),
        
        #[error("Cache error: {0}")]
        Cache(String),
        
        #[error("Authentication error: {0}")]
        Auth(String),
        
        #[error("Rate limit exceeded")]
        RateLimit,
        
        #[error("Internal server error: {0}")]
        Internal(String),
        
        #[error("Bad request: {0}")]
        BadRequest(String),
        
        #[error("Not found: {0}")]
        NotFound(String),
        
        #[error("Configuration error: {0}")]
        Config(String),
    }

    impl AppError {
        /// 获取HTTP状态码
        pub fn status_code(&self) -> http::StatusCode {
            match self {
                AppError::Validation(_) => http::StatusCode::BAD_REQUEST,
                AppError::JsonParse(_) => http::StatusCode::BAD_REQUEST,
                AppError::Schema(_) => http::StatusCode::BAD_REQUEST,
                AppError::Cache(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::Auth(_) => http::StatusCode::UNAUTHORIZED,
                AppError::RateLimit => http::StatusCode::TOO_MANY_REQUESTS,
                AppError::Internal(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
                AppError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
                AppError::NotFound(_) => http::StatusCode::NOT_FOUND,
                AppError::Config(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            }
        }

        /// 获取错误代码
        pub fn error_code(&self) -> String {
            match self {
                AppError::Validation(_) => "VALIDATION_ERROR".to_string(),
                AppError::JsonParse(_) => "JSON_PARSE_ERROR".to_string(),
                AppError::Schema(_) => "SCHEMA_ERROR".to_string(),
                AppError::Cache(_) => "CACHE_ERROR".to_string(),
                AppError::Auth(_) => "AUTH_ERROR".to_string(),
                AppError::RateLimit => "RATE_LIMIT_ERROR".to_string(),
                AppError::Internal(_) => "INTERNAL_ERROR".to_string(),
                AppError::BadRequest(_) => "BAD_REQUEST_ERROR".to_string(),
                AppError::NotFound(_) => "NOT_FOUND_ERROR".to_string(),
                AppError::Config(_) => "CONFIG_ERROR".to_string(),
            }
        }
    }

    impl From<anyhow::Error> for AppError {
        fn from(err: anyhow::Error) -> Self {
            AppError::Internal(err.to_string())
        }
    }

    impl From<serde_json::Error> for AppError {
        fn from(err: serde_json::Error) -> Self {
            AppError::JsonParse(err.to_string())
        }
    }
}

