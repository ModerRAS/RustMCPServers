use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub secret_key: String,
    pub require_auth: bool,
    pub static_tokens: Vec<String>,
    pub cors_origins: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub cache_ttl_seconds: u64,
    pub max_search_results: usize,
    pub request_timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            secret_key: "your-secret-key-change-this".to_string(),
            require_auth: false,
            static_tokens: Vec::new(),
            cors_origins: vec!["*".to_string()],
            rate_limit_per_minute: 60,
            cache_ttl_seconds: 300,
            max_search_results: 20,
            request_timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 500,
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(host) = env::var("HOST") {
            config.host = host;
        }
        
        if let Ok(port) = env::var("PORT") {
            if let Ok(port) = port.parse() {
                config.port = port;
            }
        }
        
        if let Ok(secret_key) = env::var("SECRET_KEY") {
            config.secret_key = secret_key;
        }
        
        if let Ok(require_auth) = env::var("REQUIRE_AUTH") {
            config.require_auth = require_auth.parse().unwrap_or(false);
        }
        
        if let Ok(static_tokens) = env::var("STATIC_TOKENS") {
            config.static_tokens = static_tokens
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(cors_origins) = env::var("CORS_ORIGINS") {
            config.cors_origins = cors_origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(rate_limit) = env::var("RATE_LIMIT_PER_MINUTE") {
            config.rate_limit_per_minute = rate_limit.parse().unwrap_or(60);
        }
        
        if let Ok(cache_ttl) = env::var("CACHE_TTL_SECONDS") {
            config.cache_ttl_seconds = cache_ttl.parse().unwrap_or(300);
        }
        
        if let Ok(max_results) = env::var("MAX_SEARCH_RESULTS") {
            config.max_search_results = max_results.parse().unwrap_or(20);
        }
        
        if let Ok(timeout) = env::var("REQUEST_TIMEOUT_SECONDS") {
            config.request_timeout_seconds = timeout.parse().unwrap_or(30);
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        config
    }

    pub fn auth_config(&self) -> crate::auth::AuthConfig {
        crate::auth::AuthConfig {
            secret_key: self.secret_key.clone(),
            require_auth: self.require_auth,
        }
    }
}