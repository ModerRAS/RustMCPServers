//! API密钥认证中间件

use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// API密钥信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// 密钥ID
    pub key_id: String,
    /// 密钥值（哈希存储）
    pub key_hash: String,
    /// 密钥名称
    pub name: String,
    /// 密钥描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 是否启用
    pub enabled: bool,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 速率限制（每分钟请求数）
    pub rate_limit: Option<u32>,
    /// 允许的IP地址列表
    pub allowed_ips: Option<Vec<String>>,
}

/// API密钥管理器
#[derive(Clone)]
pub struct ApiKeyManager {
    /// API密钥存储
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// 密钥前缀
    key_prefix: String,
}

impl ApiKeyManager {
    /// 创建新的API密钥管理器
    pub fn new(key_prefix: String) -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            key_prefix,
        }
    }

    /// 添加API密钥
    pub async fn add_key(&self, api_key: ApiKey) -> Result<(), ApiKeyError> {
        let mut keys = self.keys.write().await;
        keys.insert(api_key.key_id.clone(), api_key);
        Ok(())
    }

    /// 移除API密钥
    pub async fn remove_key(&self, key_id: &str) -> Result<(), ApiKeyError> {
        let mut keys = self.keys.write().await;
        keys.remove(key_id).ok_or_else(|| ApiKeyError::KeyNotFound(key_id.to_string()))?;
        Ok(())
    }

    /// 验证API密钥
    pub async fn validate_key(&self, key_value: &str) -> Result<ApiKey, ApiKeyError> {
        // 检查密钥前缀
        if !key_value.starts_with(&self.key_prefix) {
            return Err(ApiKeyError::InvalidKeyFormat);
        }

        // 提取密钥ID
        let key_id = key_value[self.key_prefix.len()..].split('_').next()
            .ok_or_else(|| ApiKeyError::InvalidKeyFormat)?;

        let keys = self.keys.read().await;
        let api_key = keys.get(key_id)
            .ok_or_else(|| ApiKeyError::KeyNotFound(key_id.to_string()))?;

        // 检查密钥是否启用
        if !api_key.enabled {
            return Err(ApiKeyError::KeyDisabled);
        }

        // 检查密钥是否过期
        if let Some(expires_at) = api_key.expires_at {
            if chrono::Utc::now() > expires_at {
                return Err(ApiKeyError::KeyExpired);
            }
        }

        // 验证密钥值（简化版本，实际应该使用bcrypt等安全哈希）
        // 注意：这里应该使用安全的密码哈希验证
        if !self.verify_key_hash(key_value, &api_key.key_hash) {
            return Err(ApiKeyError::InvalidKey);
        }

        Ok(api_key.clone())
    }

    /// 验证密钥哈希（简化版本）
    fn verify_key_hash(&self, key_value: &str, key_hash: &str) -> bool {
        // 简化版本：实际应该使用bcrypt等安全哈希
        // 这里为了演示，使用简单的比较
        key_hash == key_value
    }

    /// 检查权限
    pub fn check_permission(&self, api_key: &ApiKey, required_permission: &str) -> bool {
        api_key.permissions.contains(&required_permission.to_string())
    }

    /// 检查IP白名单
    pub fn check_ip_whitelist(&self, api_key: &ApiKey, client_ip: &str) -> bool {
        if let Some(allowed_ips) = &api_key.allowed_ips {
            if allowed_ips.is_empty() {
                return true;
            }
            allowed_ips.contains(&client_ip.to_string())
        } else {
            true
        }
    }

    /// 获取所有密钥
    pub async fn get_all_keys(&self) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        keys.values().cloned().collect()
    }

    /// 生成新的API密钥
    pub fn generate_key(&self, key_id: &str, name: String) -> String {
        format!("{}_{}", self.key_prefix, key_id)
    }
}

/// API密钥认证中间件
pub struct ApiKeyAuthLayer {
    /// API密钥管理器
    key_manager: Arc<ApiKeyManager>,
    /// 是否启用严格模式
    strict_mode: bool,
}

impl ApiKeyAuthLayer {
    /// 创建新的API密钥认证中间件
    pub fn new(key_manager: Arc<ApiKeyManager>) -> Self {
        Self {
            key_manager,
            strict_mode: true,
        }
    }

    /// 设置严格模式
    pub fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    /// 从请求头提取API密钥
    fn extract_api_key(&self, request: &Request) -> Option<String> {
        // 1. 检查Authorization头 (Bearer token)
        if let Some(auth_header) = request.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    return Some(auth_str[7..].to_string());
                }
            }
        }

        // 2. 检查X-API-Key头
        if let Some(api_key_header) = request.headers().get("X-API-Key") {
            if let Ok(api_key) = api_key_header.to_str() {
                return Some(api_key.to_string());
            }
        }

        // 3. 检查查询参数
        if let Some(query) = request.uri().query() {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();
            if let Some(api_key) = params.get("api_key") {
                return Some(api_key.clone());
            }
        }

        None
    }

    /// 获取客户端IP地址
    fn get_client_ip(&self, request: &Request) -> Option<String> {
        // 1. 检查X-Forwarded-For头
        if let Some(x_forwarded_for) = request.headers().get("X-Forwarded-For") {
            if let Ok(xff_str) = x_forwarded_for.to_str() {
                let first_ip = xff_str.split(',').next()?.trim();
                return Some(first_ip.to_string());
            }
        }

        // 2. 检查X-Real-IP头
        if let Some(x_real_ip) = request.headers().get("X-Real-IP") {
            if let Ok(real_ip) = x_real_ip.to_str() {
                return Some(real_ip.to_string());
            }
        }

        None
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for ApiKeyAuthLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        // 提取API密钥
        let api_key_value = match self.extract_api_key(&req) {
            Some(key) => key,
            None => {
                warn!("Missing API key in request");
                return Ok(create_api_key_error_response("Missing API key", StatusCode::UNAUTHORIZED));
            }
        };

        // 验证API密钥
        let api_key = match self.key_manager.validate_key(&api_key_value).await {
            Ok(key) => {
                debug!("API key validated successfully: {}", key.key_id);
                key
            }
            Err(e) => {
                warn!("API key validation failed: {}", e);
                return Ok(create_api_key_error_response(&format!("Invalid API key: {}", e), StatusCode::UNAUTHORIZED));
            }
        };

        // 检查IP白名单
        if let Some(client_ip) = self.get_client_ip(&req) {
            if !self.key_manager.check_ip_whitelist(&api_key, &client_ip) {
                warn!("IP address not in whitelist: {}", client_ip);
                return Ok(create_api_key_error_response("IP address not allowed", StatusCode::FORBIDDEN));
            }
        }

        // 将API密钥信息添加到请求扩展中
        let mut req = req;
        req.extensions_mut().insert(api_key);

        // 继续处理请求
        let response = next.run(req).await;
        Ok(response)
    }
}

/// API密钥认证错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiKeyError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid key format")]
    InvalidKeyFormat,
    
    #[error("Invalid key")]
    InvalidKey,
    
    #[error("Key disabled")]
    KeyDisabled,
    
    #[error("Key expired")]
    KeyExpired,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("IP address not allowed")]
    IpNotAllowed,
}

/// API密钥信息提取器
pub struct ApiKeyInfo {
    /// 密钥ID
    pub key_id: String,
    /// 密钥名称
    pub name: String,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 速率限制
    pub rate_limit: Option<u32>,
}

/// 从请求扩展中提取API密钥信息
pub fn extract_api_key_info(req: &Request) -> Result<ApiKeyInfo, ApiKeyError> {
    req.extensions()
        .get::<ApiKey>()
        .ok_or_else(|| ApiKeyError::KeyNotFound("No API key found in request".to_string()))
        .map(|api_key| ApiKeyInfo {
            key_id: api_key.key_id.clone(),
            name: api_key.name.clone(),
            permissions: api_key.permissions.clone(),
            rate_limit: api_key.rate_limit,
        })
}

/// 权限检查中间件
pub async fn require_api_key_permission(
    permission: &str,
    req: Request,
    next: Next,
) -> Result<Response, axum::Error> {
    match extract_api_key_info(&req) {
        Ok(api_key_info) => {
            if api_key_info.permissions.contains(&permission.to_string()) {
                debug!("API key {} has permission: {}", api_key_info.key_id, permission);
                Ok(next.run(req).await)
            } else {
                warn!("API key {} lacks permission: {}", api_key_info.key_id, permission);
                Ok(create_api_key_error_response(
                    &format!("Insufficient permissions: required {}", permission),
                    StatusCode::FORBIDDEN,
                ))
            }
        }
        Err(e) => {
            warn!("API key authentication failed: {}", e);
            Ok(create_api_key_error_response(&format!("Authentication failed: {}", e), StatusCode::UNAUTHORIZED))
        }
    }
}

/// 创建API密钥错误响应
fn create_api_key_error_response(message: &str, status: StatusCode) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": "API_KEY_ERROR",
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body.to_string().into())
        .unwrap()
}

/// API密钥生成器
pub struct ApiKeyGenerator {
    prefix: String,
    key_length: usize,
}

impl ApiKeyGenerator {
    /// 创建新的API密钥生成器
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            key_length: 32,
        }
    }

    /// 设置密钥长度
    pub fn with_key_length(mut self, length: usize) -> Self {
        self.key_length = length;
        self
    }

    /// 生成新的API密钥
    pub fn generate(&self, key_id: &str) -> String {
        let random_part: String = uuid::Uuid::new_v4()
            .to_string()
            .replace('-', "")
            .chars()
            .take(self.key_length)
            .collect();
        
        format!("{}_{}_{}", self.prefix, key_id, random_part)
    }

    /// 生成安全的API密钥
    pub fn generate_secure(&self, key_id: &str) -> (String, String) {
        let api_key = self.generate(key_id);
        let key_hash = bcrypt::hash(&api_key, bcrypt::DEFAULT_COST)
            .unwrap_or_else(|_| api_key.clone());
        
        (api_key, key_hash)
    }
}

impl Default for ApiKeyGenerator {
    fn default() -> Self {
        Self::new("json-validator".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_api_key_manager() {
        let manager = ApiKeyManager::new("test".to_string());
        
        let api_key = ApiKey {
            key_id: "test_key".to_string(),
            key_hash: "test_hash".to_string(),
            name: "Test Key".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            expires_at: None,
            enabled: true,
            permissions: vec!["read".to_string()],
            rate_limit: Some(100),
            allowed_ips: None,
        };

        manager.add_key(api_key.clone()).await.unwrap();
        
        let validated_key = manager.validate_key("test_test_key").await.unwrap();
        assert_eq!(validated_key.key_id, "test_key");
    }

    #[tokio::test]
    async fn test_api_key_generator() {
        let generator = ApiKeyGenerator::new("test".to_string());
        let key = generator.generate("123");
        assert!(key.starts_with("test_123_"));
        assert!(key.len() > 10);
    }

    #[test]
    fn test_api_key_info_extraction() {
        let api_key = ApiKey {
            key_id: "test".to_string(),
            key_hash: "hash".to_string(),
            name: "Test".to_string(),
            description: None,
            created_at: chrono::Utc::now(),
            expires_at: None,
            enabled: true,
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limit: Some(100),
            allowed_ips: None,
        };

        let mut request = Request::new(Body::empty());
        request.extensions_mut().insert(api_key);

        let info = extract_api_key_info(&request).unwrap();
        assert_eq!(info.key_id, "test");
        assert_eq!(info.permissions, vec!["read", "write"]);
    }
}