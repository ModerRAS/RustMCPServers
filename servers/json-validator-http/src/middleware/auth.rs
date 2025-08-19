//! 认证中间件

use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// 用户ID
    pub sub: String,
    /// 用户名
    pub username: String,
    /// 角色列表
    pub roles: Vec<String>,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 签发时间
    pub iat: usize,
    /// 过期时间
    pub exp: usize,
    /// 签发者
    pub iss: String,
}

/// 认证中间件层
pub struct AuthLayer {
    /// 编码密钥
    encoding_key: EncodingKey,
    /// 解码密钥
    decoding_key: DecodingKey,
    /// 验证配置
    validation: Validation,
    /// 签发者
    issuer: String,
}

impl AuthLayer {
    /// 创建新的认证中间件
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["json-validator-server"]);
        validation.validate_exp = true;
        
        Self {
            encoding_key,
            decoding_key,
            validation,
            issuer: "json-validator-server".to_string(),
        }
    }
    
    /// 生成JWT令牌
    pub fn generate_token(&self, user_id: &str, username: &str, roles: Vec<String>, permissions: Vec<String>) -> Result<String, AuthError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthError::TokenGeneration("Failed to get current time".to_string()))?;
        
        let iat = now.as_secs() as usize;
        let exp = (now + Duration::from_secs(86400)).as_secs() as usize; // 24小时
        
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles,
            permissions,
            iat,
            exp,
            iss: self.issuer.clone(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGeneration(format!("Failed to encode token: {}", e)))
    }
    
    /// 验证JWT令牌
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::InvalidToken(format!("Failed to decode token: {}", e)))
    }
    
    /// 检查权限
    pub fn check_permission(&self, claims: &Claims, required_permission: &str) -> bool {
        claims.permissions.contains(&required_permission.to_string())
    }
    
    /// 检查角色
    pub fn check_role(&self, claims: &Claims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string())
    }
}

/// 认证中间件实现
#[axum::async_trait]
impl<S> axum::middleware::Next<S> for AuthLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        // 从请求头中获取Authorization令牌
        let auth_header = req.headers().get("Authorization");
        
        match auth_header {
            Some(header) => {
                let auth_str = header.to_str().unwrap_or("");
                
                if !auth_str.starts_with("Bearer ") {
                    warn!("Invalid authorization header format");
                    return Ok(create_auth_error_response("Invalid authorization header format"));
                }
                
                let token = &auth_str[7..]; // 移除"Bearer "前缀
                
                match self.validate_token(token) {
                    Ok(claims) => {
                        debug!("User {} authenticated successfully", claims.username);
                        
                        // 将claims添加到请求扩展中
                        let mut req = req;
                        req.extensions_mut().insert(claims);
                        
                        // 继续处理请求
                        let response = next.run(req).await;
                        Ok(response)
                    }
                    Err(e) => {
                        warn!("Token validation failed: {}", e);
                        Ok(create_auth_error_response(&format!("Invalid token: {}", e)))
                    }
                }
            }
            None => {
                warn!("Missing authorization header");
                Ok(create_auth_error_response("Missing authorization header"))
            }
        }
    }
}

/// 认证错误类型
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Token generation error: {0}")]
    TokenGeneration(String),
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Missing token")]
    MissingToken,
    
    #[error("Expired token")]
    ExpiredToken,
    
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    
    #[error("Invalid role")]
    InvalidRole,
}

/// 认证提取器
pub struct AuthenticatedUser {
    /// 用户ID
    pub user_id: String,
    /// 用户名
    pub username: String,
    /// 角色列表
    pub roles: Vec<String>,
    /// 权限列表
    pub permissions: Vec<String>,
}

/// 从请求扩展中提取认证用户
pub fn extract_authenticated_user(req: &Request) -> Result<AuthenticatedUser, AuthError> {
    req.extensions()
        .get::<Claims>()
        .ok_or_else(|| AuthError::MissingToken)
        .map(|claims| AuthenticatedUser {
            user_id: claims.sub.clone(),
            username: claims.username.clone(),
            roles: claims.roles.clone(),
            permissions: claims.permissions.clone(),
        })
}

/// 权限检查中间件
pub async fn require_permission(
    permission: &str,
    req: Request,
    next: Next,
) -> Result<Response, axum::Error> {
    match extract_authenticated_user(&req) {
        Ok(user) => {
            if user.permissions.contains(&permission.to_string()) {
                debug!("User {} has permission: {}", user.username, permission);
                Ok(next.run(req).await)
            } else {
                warn!("User {} lacks permission: {}", user.username, permission);
                Ok(create_permission_error_response(
                    &format!("Insufficient permissions: required {}", permission),
                ))
            }
        }
        Err(e) => {
            warn!("Authentication failed: {}", e);
            Ok(create_auth_error_response(&format!("Authentication failed: {}", e)))
        }
    }
}

/// 角色检查中间件
pub async fn require_role(
    role: &str,
    req: Request,
    next: Next,
) -> Result<Response, axum::Error> {
    match extract_authenticated_user(&req) {
        Ok(user) => {
            if user.roles.contains(&role.to_string()) {
                debug!("User {} has role: {}", user.username, role);
                Ok(next.run(req).await)
            } else {
                warn!("User {} lacks role: {}", user.username, role);
                Ok(create_permission_error_response(
                    &format!("Insufficient role: required {}", role),
                ))
            }
        }
        Err(e) => {
            warn!("Authentication failed: {}", e);
            Ok(create_auth_error_response(&format!("Authentication failed: {}", e)))
        }
    }
}

/// 创建认证错误响应
fn create_auth_error_response(message: &str) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": "AUTH_ERROR",
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });
    
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(body.to_string().into())
        .unwrap()
}

/// 创建权限错误响应
fn create_permission_error_response(message: &str) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": "PERMISSION_ERROR",
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });
    
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("content-type", "application/json")
        .body(body.to_string().into())
        .unwrap()
}

/// 权限检查提取器
pub struct PermissionCheck(pub String);

/// 角色检查提取器
pub struct RoleCheck(pub String);

/// 权限检查中间件工厂
pub fn require_permission_middleware(permission: String) -> impl axum::middleware::FromFn<()> {
    move |req: Request, next: Next| require_permission(&permission, req, next)
}

/// 角色检查中间件工厂
pub fn require_role_middleware(role: String) -> impl axum::middleware::FromFn<()> {
    move |req: Request, next: Next| require_role(&role, req, next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[test]
    fn test_auth_layer_creation() {
        let auth_layer = AuthLayer::new("test-secret");
        assert_eq!(auth_layer.issuer, "json-validator-server");
    }

    #[test]
    fn test_token_generation() {
        let auth_layer = AuthLayer::new("test-secret");
        let token = auth_layer.generate_token(
            "user123",
            "testuser",
            vec!["user".to_string()],
            vec!["read".to_string()],
        );
        
        assert!(token.is_ok());
    }

    #[test]
    fn test_token_validation() {
        let auth_layer = AuthLayer::new("test-secret");
        let token = auth_layer.generate_token(
            "user123",
            "testuser",
            vec!["user".to_string()],
            vec!["read".to_string()],
        ).unwrap();
        
        let claims = auth_layer.validate_token(&token);
        assert!(claims.is_ok());
        
        let claims = claims.unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.username, "testuser");
        assert!(claims.roles.contains(&"user".to_string()));
        assert!(claims.permissions.contains(&"read".to_string()));
    }

    #[test]
    fn test_permission_check() {
        let auth_layer = AuthLayer::new("test-secret");
        let claims = Claims {
            sub: "user123".to_string(),
            username: "testuser".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            iat: 1000,
            exp: 2000,
            iss: "json-validator-server".to_string(),
        };
        
        assert!(auth_layer.check_permission(&claims, "read"));
        assert!(auth_layer.check_permission(&claims, "write"));
        assert!(!auth_layer.check_permission(&claims, "delete"));
    }

    #[test]
    fn test_role_check() {
        let auth_layer = AuthLayer::new("test-secret");
        let claims = Claims {
            sub: "user123".to_string(),
            username: "testuser".to_string(),
            roles: vec!["user".to_string(), "admin".to_string()],
            permissions: vec!["read".to_string()],
            iat: 1000,
            exp: 2000,
            iss: "json-validator-server".to_string(),
        };
        
        assert!(auth_layer.check_role(&claims, "user"));
        assert!(auth_layer.check_role(&claims, "admin"));
        assert!(!auth_layer.check_role(&claims, "superuser"));
    }
}