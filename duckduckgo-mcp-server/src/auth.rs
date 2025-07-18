use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub secret_key: String,
    pub require_auth: bool,
}

#[derive(Debug, Clone)]
pub struct AuthState {
    pub config: AuthConfig,
    pub valid_tokens: Arc<RwLock<Vec<String>>>,
}

impl AuthState {
    pub fn new(secret_key: String, require_auth: bool) -> Self {
        Self {
            config: AuthConfig {
                secret_key,
                require_auth,
            },
            valid_tokens: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn generate_token(&self, username: &str) -> Result<String> {
        let now = chrono::Utc::now().timestamp() as usize;
        let exp = now + 3600; // Token expires in 1 hour

        let claims = Claims {
            sub: username.to_string(),
            exp,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.secret_key.as_bytes()),
        )?;

        Ok(token)
    }

    pub async fn validate_token(&self, token: &str) -> Result<bool> {
        if !self.config.require_auth {
            return Ok(true);
        }

        // Check if it's a static token
        {
            let valid_tokens = self.valid_tokens.read().await;
            if valid_tokens.contains(&token.to_string()) {
                return Ok(true);
            }
        }

        // Check if it's a JWT token
        let validation = Validation::default();
        let result = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.secret_key.as_bytes()),
            &validation,
        );

        match result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn add_static_token(&self, token: String) {
        let mut valid_tokens = self.valid_tokens.write().await;
        if !valid_tokens.contains(&token) {
            valid_tokens.push(token);
        }
    }

    pub async fn remove_static_token(&self, token: &str) {
        let mut valid_tokens = self.valid_tokens.write().await;
        valid_tokens.retain(|t| t != token);
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<crate::mcp_handler::McpState>>,
    headers: HeaderMap,
    mut request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_state = &state.auth;
    if !auth_state.config.require_auth {
        return Ok(next.run(request).await);
    }

    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .or_else(|| headers.get("X-API-Key").and_then(|h| h.to_str().ok()));

    let token = match auth_header {
        Some(header) => {
            if let Some(stripped) = header.strip_prefix("Bearer ") {
                stripped
            } else {
                header
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    match auth_state.validate_token(token).await {
        Ok(true) => {
            // Add user info to request extensions
            request.extensions_mut().insert(token.to_string());
            Ok(next.run(request).await)
        }
        Ok(false) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    pub token: String,
}
