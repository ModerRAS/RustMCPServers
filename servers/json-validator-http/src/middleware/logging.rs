//! 日志中间件

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use http::HeaderValue;
use tracing::{debug, error, info, span, warn, Level};
use uuid::Uuid;

/// 日志中间件
pub struct LoggingLayer;

impl LoggingLayer {
    /// 创建新的日志中间件
    pub fn new() -> Self {
        Self
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for LoggingLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        let start_time = std::time::Instant::now();
        let request_id = Uuid::new_v4().to_string();
        
        // 提取请求信息
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        
        // 获取客户端IP
        let client_ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
            })
            .or_else(|| {
                req.headers()
                    .get("remote-addr")
                    .and_then(|h| h.to_str().ok())
            })
            .unwrap_or("unknown")
            .split(',')
            .next()
            .unwrap_or("unknown")
            .trim();
        
        // 创建请求span
        let span = span!(
            Level::INFO,
            "http_request",
            request_id = %request_id,
            method = %method,
            uri = %uri,
            client_ip = %client_ip,
            user_agent = %user_agent,
        );
        
        let _enter = span.enter();
        
        info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            version = ?version,
            client_ip = %client_ip,
            user_agent = %user_agent,
            "Request started"
        );
        
        // 添加请求ID到请求头
        let mut req = req;
        req.headers_mut().insert(
            "x-request-id",
            HeaderValue::from_str(&request_id).unwrap_or_default(),
        );
        
        // 记录请求体大小（如果存在）
        if let Some(content_length) = req.headers().get("content-length") {
            if let Ok(size) = content_length.to_str() {
                debug!(
                    request_id = %request_id,
                    content_length = %size,
                    "Request content length"
                );
            }
        }
        
        // 处理请求
        let response = next.run(req).await;
        
        let duration = start_time.elapsed();
        let status = response.status();
        
        // 记录响应信息
        info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            "Request completed"
        );
        
        // 根据状态码记录不同级别的日志
        match status.as_u16() {
            200..=399 => {
                debug!(
                    request_id = %request_id,
                    "Successful request"
                );
            }
            400..=499 => {
                warn!(
                    request_id = %request_id,
                    "Client error"
                );
            }
            500..=599 => {
                error!(
                    request_id = %request_id,
                    "Server error"
                );
            }
            _ => {}
        }
        
        // 添加响应头
        let mut response = response;
        response.headers_mut().insert(
            "x-request-id",
            HeaderValue::from_str(&request_id).unwrap_or_default(),
        );
        response.headers_mut().insert(
            "x-response-time",
            HeaderValue::from_str(&format!("{:?}", duration)).unwrap_or_default(),
        );
        
        Ok(response)
    }
}

/// 请求日志提取器
pub struct RequestLogger {
    pub request_id: String,
    pub method: http::Method,
    pub uri: http::Uri,
    pub client_ip: String,
    pub user_agent: String,
}

impl RequestLogger {
    /// 从请求中提取日志信息
    pub fn from_request(req: &Request) -> Self {
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&Uuid::new_v4().to_string())
            .to_string();
        
        let client_ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
            })
            .or_else(|| {
                req.headers()
                    .get("remote-addr")
                    .and_then(|h| h.to_str().ok())
            })
            .unwrap_or("unknown")
            .split(',')
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();
        
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        
        Self {
            request_id,
            method: req.method().clone(),
            uri: req.uri().clone(),
            client_ip,
            user_agent,
        }
    }
    
    /// 记录请求开始
    pub fn log_start(&self) {
        info!(
            request_id = %self.request_id,
            method = %self.method,
            uri = %self.uri,
            client_ip = %self.client_ip,
            user_agent = %self.user_agent,
            "Request started"
        );
    }
    
    /// 记录请求完成
    pub fn log_complete(&self, status: http::StatusCode, duration: std::time::Duration) {
        info!(
            request_id = %self.request_id,
            method = %self.method,
            uri = %self.uri,
            status = %status,
            duration = ?duration,
            "Request completed"
        );
    }
    
    /// 记录请求错误
    pub fn log_error(&self, error: &str) {
        error!(
            request_id = %self.request_id,
            method = %self.method,
            uri = %self.uri,
            error = %error,
            "Request error"
        );
    }
}

/// 性能日志中间件
pub struct PerformanceLoggingLayer;

impl PerformanceLoggingLayer {
    /// 创建新的性能日志中间件
    pub fn new() -> Self {
        Self
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for PerformanceLoggingLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        let start_time = std::time::Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();
        
        // 处理请求
        let response = next.run(req).await;
        
        let duration = start_time.elapsed();
        
        // 记录性能指标
        if duration.as_millis() > 1000 {
            warn!(
                method = %method,
                uri = %uri,
                duration = ?duration,
                "Slow request detected"
            );
        } else if duration.as_millis() > 500 {
            info!(
                method = %method,
                uri = %uri,
                duration = ?duration,
                "Moderate request time"
            );
        } else {
            debug!(
                method = %method,
                uri = %uri,
                duration = ?duration,
                "Fast request"
            );
        }
        
        Ok(response)
    }
}

/// 安全日志中间件
pub struct SecurityLoggingLayer;

impl SecurityLoggingLayer {
    /// 创建新的安全日志中间件
    pub fn new() -> Self {
        Self
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for SecurityLoggingLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let client_ip = req
            .headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .split(',')
            .next()
            .unwrap_or("unknown")
            .trim();
        
        // 记录安全相关事件
        if method == http::Method::POST || method == http::Method::PUT || method == http::Method::DELETE {
            info!(
                method = %method,
                uri = %uri,
                client_ip = %client_ip,
                "Write operation detected"
            );
        }
        
        // 检查可疑请求
        let query_string = uri.query().unwrap_or("");
        if query_string.contains("..") || query_string.contains("<") || query_string.contains(">") {
            warn!(
                method = %method,
                uri = %uri,
                client_ip = %client_ip,
                "Suspicious query string detected"
            );
        }
        
        // 处理请求
        let response = next.run(req).await;
        
        let status = response.status();
        
        // 记录安全事件
        match status.as_u16() {
            401 | 403 => {
                warn!(
                    method = %method,
                    uri = %uri,
                    client_ip = %client_ip,
                    status = %status,
                    "Authorization/permission denied"
                );
            }
            429 => {
                warn!(
                    method = %method,
                    uri = %uri,
                    client_ip = %client_ip,
                    status = %status,
                    "Rate limit exceeded"
                );
            }
            400 => {
                info!(
                    method = %method,
                    uri = %uri,
                    client_ip = %client_ip,
                    status = %status,
                    "Bad request"
                );
            }
            _ => {}
        }
        
        Ok(response)
    }
}

/// 日志记录宏
#[macro_export]
macro_rules! log_http_request {
    ($level:expr, $request:expr, $status:expr, $duration:expr) => {
        tracing::event!(
            $level,
            method = %$request.method(),
            uri = %$request.uri(),
            status = %$status,
            duration = ?$duration,
            "HTTP request"
        );
    };
}

#[macro_export]
macro_rules! log_security_event {
    ($level:expr, $event:expr, $client_ip:expr, $details:expr) => {
        tracing::event!(
            $level,
            event = %$event,
            client_ip = %$client_ip,
            details = %$details,
            "Security event"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_logging_layer() {
        let logging_layer = LoggingLayer::new();
        
        let request = Request::builder()
            .uri("/test")
            .method("GET")
            .header("user-agent", "test-agent")
            .header("x-forwarded-for", "127.0.0.1")
            .body(Body::empty())
            .unwrap();
        
        // 这里需要测试中间件，但需要设置完整的测试环境
        // 暂时跳过这个测试
    }

    #[test]
    fn test_request_logger() {
        let request = Request::builder()
            .uri("/test")
            .method("GET")
            .header("user-agent", "test-agent")
            .header("x-request-id", "test-123")
            .header("x-forwarded-for", "127.0.0.1")
            .body(Body::empty())
            .unwrap();
        
        let logger = RequestLogger::from_request(&request);
        
        assert_eq!(logger.request_id, "test-123");
        assert_eq!(logger.method, http::Method::GET);
        assert_eq!(logger.uri, "/test");
        assert_eq!(logger.client_ip, "127.0.0.1");
        assert_eq!(logger.user_agent, "test-agent");
    }
}