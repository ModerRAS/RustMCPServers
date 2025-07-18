use crate::auth::{AuthResponse, AuthState, LoginRequest, TokenRequest};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn login_handler(
    State(auth_state): State<Arc<AuthState>>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Simple username/password validation
    // In production, this should check against a database
    if login_request.username == "admin" && login_request.password == "password" {
        match auth_state.generate_token(&login_request.username) {
            Ok(token) => {
                let response = json!({
                    "success": true,
                    "token": token,
                    "expires_in": 3600
                });
                Ok(Json(response))
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn validate_token_handler(
    State(auth_state): State<Arc<AuthState>>,
    Json(token_request): Json<TokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match auth_state.validate_token(&token_request.token).await {
        Ok(true) => {
            let response = json!({
                "valid": true,
                "message": "Token is valid"
            });
            Ok(Json(response))
        }
        Ok(false) => {
            let response = json!({
                "valid": false,
                "message": "Token is invalid or expired"
            });
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn add_static_token_handler(
    State(auth_state): State<Arc<AuthState>>,
    Json(token_request): Json<TokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    auth_state.add_static_token(token_request.token).await;
    
    let response = json!({
        "success": true,
        "message": "Token added successfully"
    });
    Ok(Json(response))
}

pub async fn remove_static_token_handler(
    State(auth_state): State<Arc<AuthState>>,
    Json(token_request): Json<TokenRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    auth_state.remove_static_token(&token_request.token).await;
    
    let response = json!({
        "success": true,
        "message": "Token removed successfully"
    });
    Ok(Json(response))
}

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    let response = json!({
        "status": "healthy",
        "service": "duckduckgo-mcp-server",
        "version": "0.1.0"
    });
    Ok(Json(response))
}
