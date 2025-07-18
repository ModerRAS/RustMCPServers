use crate::auth_service::{AuthService, LoginRequest, TokenResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;

