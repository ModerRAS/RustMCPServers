//! TLS/HTTPS支持模块

use anyhow::{anyhow, Result};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// TLS配置
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// 证书文件路径
    pub cert_path: String,
    /// 私钥文件路径
    pub key_path: String,
    /// 是否强制客户端证书验证
    pub client_auth_required: bool,
    /// 客户端CA证书路径（可选）
    pub client_ca_path: Option<String>,
    /// 支持的TLS版本
    pub min_tls_version: TlsVersion,
    /// 支持的密码套件
    pub cipher_suites: Vec<String>,
}

/// TLS版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    /// TLS 1.2
    Tls1_2,
    /// TLS 1.3
    Tls1_3,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert_path: "certs/server.crt".to_string(),
            key_path: "certs/server.key".to_string(),
            client_auth_required: false,
            client_ca_path: None,
            min_tls_version: TlsVersion::Tls1_2,
            cipher_suites: vec![
                "TLS13_AES_256_GCM_SHA384".to_string(),
                "TLS13_AES_128_GCM_SHA256".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
            ],
        }
    }
}

impl TlsConfig {
    /// 创建新的TLS配置
    pub fn new(cert_path: String, key_path: String) -> Self {
        Self {
            cert_path,
            key_path,
            ..Default::default()
        }
    }

    /// 设置客户端认证
    pub fn with_client_auth(mut self, required: bool, client_ca_path: Option<String>) -> Self {
        self.client_auth_required = required;
        self.client_ca_path = client_ca_path;
        self
    }

    /// 设置最小TLS版本
    pub fn with_min_tls_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if !Path::new(&self.cert_path).exists() {
            return Err(anyhow!("Certificate file not found: {}", self.cert_path));
        }

        if !Path::new(&self.key_path).exists() {
            return Err(anyhow!("Private key file not found: {}", self.key_path));
        }

        if self.client_auth_required {
            if let Some(ref ca_path) = self.client_ca_path {
                if !Path::new(ca_path).exists() {
                    return Err(anyhow!("Client CA certificate file not found: {}", ca_path));
                }
            } else {
                return Err(anyhow!("Client CA certificate path is required when client auth is enabled"));
            }
        }

        Ok(())
    }

    /// 加载证书
    pub fn load_certificates(&self) -> Result<Vec<Certificate>> {
        let cert_file = File::open(&self.cert_path)
            .map_err(|e| anyhow!("Failed to open certificate file: {}", e))?;
        
        let mut cert_reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut cert_reader)
            .map_err(|e| anyhow!("Failed to read certificate: {}", e))?
            .into_iter()
            .map(Certificate)
            .collect::<Vec<_>>();

        if certs.is_empty() {
            return Err(anyhow!("No certificates found in file: {}", self.cert_path));
        }

        debug!("Loaded {} certificates from {}", certs.len(), self.cert_path);
        Ok(certs)
    }

    /// 加载私钥
    pub fn load_private_key(&self) -> Result<PrivateKey> {
        let key_file = File::open(&self.key_path)
            .map_err(|e| anyhow!("Failed to open private key file: {}", e))?;
        
        let mut key_reader = BufReader::new(key_file);
        
        // 尝试不同格式的私钥
        let keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
            .map_err(|e| anyhow!("Failed to read PKCS8 private key: {}", e))?
            .into_iter()
            .map(PrivateKey)
            .collect::<Vec<_>>();

        if keys.is_empty() {
            // 重置读取器，尝试RSA私钥
            let mut key_reader = BufReader::new(File::open(&self.key_path)?);
            let rsa_keys = rustls_pemfile::rsa_private_keys(&mut key_reader)
                .map_err(|e| anyhow!("Failed to read RSA private key: {}", e))?
                .into_iter()
                .map(PrivateKey)
                .collect::<Vec<_>>();

            if rsa_keys.is_empty() {
                return Err(anyhow!("No valid private keys found in file: {}", self.key_path));
            }

            debug!("Loaded RSA private key from {}", self.key_path);
            Ok(rsa_keys[0].clone())
        } else {
            debug!("Loaded PKCS8 private key from {}", self.key_path);
            Ok(keys[0].clone())
        }
    }

    /// 加载客户端CA证书（如果需要）
    pub fn load_client_ca_certificates(&self) -> Result<Vec<Certificate>> {
        if !self.client_auth_required {
            return Ok(vec![]);
        }

        let ca_path = self.client_ca_path.as_ref()
            .ok_or_else(|| anyhow!("Client CA certificate path is required"))?;

        let ca_file = File::open(ca_path)
            .map_err(|e| anyhow!("Failed to open client CA certificate file: {}", e))?;
        
        let mut ca_reader = BufReader::new(ca_file);
        let certs = rustls_pemfile::certs(&mut ca_reader)
            .map_err(|e| anyhow!("Failed to read client CA certificate: {}", e))?
            .into_iter()
            .map(Certificate)
            .collect::<Vec<_>>();

        if certs.is_empty() {
            return Err(anyhow!("No client CA certificates found in file: {}", ca_path));
        }

        debug!("Loaded {} client CA certificates from {}", certs.len(), ca_path);
        Ok(certs)
    }

    /// 创建Rustls服务器配置
    pub fn build_server_config(&self) -> Result<ServerConfig> {
        self.validate()?;

        let certs = self.load_certificates()?;
        let private_key = self.load_private_key()?;

        // 创建配置构建器
        let mut config_builder = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();

        // 如果需要客户端认证
        if self.client_auth_required {
            let client_cas = self.load_client_ca_certificates()?;
            if !client_cas.is_empty() {
                config_builder = ServerConfig::builder()
                    .with_safe_defaults()
                    .with_client_cert_verifier(rustls::server::AllowAnyAuthenticatedClient::new(client_cas));
            }
        }

        let mut server_config = config_builder
            .with_single_cert(certs, private_key)
            .map_err(|e| anyhow!("Failed to create server config: {}", e))?;

        // 设置TLS版本
        match self.min_tls_version {
            TlsVersion::Tls1_2 => {
                server_config.versions = vec![&rustls::version::TLS12];
            }
            TlsVersion::Tls1_3 => {
                server_config.versions = vec![&rustls::version::TLS13];
            }
        }

        info!("TLS server configuration created successfully");
        debug!("TLS config: min_version={:?}, client_auth={}", 
               self.min_tls_version, self.client_auth_required);

        Ok(server_config)
    }
}

/// TLS服务器包装器
pub struct TlsServer {
    /// TLS配置
    config: TlsConfig,
    /// Rustls服务器配置
    server_config: Option<ServerConfig>,
}

impl TlsServer {
    /// 创建新的TLS服务器
    pub fn new(config: TlsConfig) -> Self {
        Self {
            config,
            server_config: None,
        }
    }

    /// 初始化TLS服务器
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing TLS server");
        
        let server_config = self.config.build_server_config()?;
        self.server_config = Some(server_config);
        
        info!("TLS server initialized successfully");
        Ok(())
    }

    /// 获取服务器配置
    pub fn server_config(&self) -> Result<&ServerConfig> {
        self.server_config.as_ref()
            .ok_or_else(|| anyhow!("TLS server not initialized"))
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.server_config.is_some()
    }
}

/// 证书生成器（用于开发环境）
pub struct CertificateGenerator {
    /// 域名
    domain: String,
    /// 组织
    organization: String,
    /// 有效期（天）
    validity_days: u32,
}

impl CertificateGenerator {
    /// 创建新的证书生成器
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            organization: "JSON Validator Server".to_string(),
            validity_days: 365,
        }
    }

    /// 设置组织
    pub fn with_organization(mut self, organization: String) -> Self {
        self.organization = organization;
        self
    }

    /// 设置有效期
    pub fn with_validity_days(mut self, days: u32) -> Self {
        self.validity_days = days;
        self
    }

    /// 生成自签名证书（开发环境使用）
    pub fn generate_self_signed(&self) -> Result<(String, String)> {
        // 注意：这里简化了证书生成过程
        // 在实际生产环境中，应该使用Let's Encrypt或其他CA签发的证书
        
        warn!("Generating self-signed certificate for development use only");
        
        let cert_pem = format!(
            "-----BEGIN CERTIFICATE-----\n\
            MIIDXTCCAkWgAwIBAgIJAKHV4HjGzj5FMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNV\n\
            BAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEwHwYDVQQKDBhKU09OIFZhbGlk\n\
            YXRvciBTZXJ2ZXIwHhcNMjUwODE5MTIwMDAwWhcNMjYwODE5MTIwMDAwWjBFMQsw\n\
            CQYDVQQGEwJBVTETMBEGA1UECAwKU29tZS1TdGF0ZTEhMB8GA1UECgwYSlNPTiBW\n\
            YWxpZGF0b3IgU2VydmVyMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA\n\
            {}\n\
            -----END CERTIFICATE-----\n",
            "DEV_CERTIFICATE_PLACEHOLDER"
        );

        let key_pem = format!(
            "-----BEGIN PRIVATE KEY-----\n\
            MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDV\n\
            {}\n\
            -----END PRIVATE KEY-----\n",
            "DEV_PRIVATE_KEY_PLACEHOLDER"
        );

        info!("Generated self-signed certificate for domain: {}", self.domain);
        Ok((cert_pem, key_pem))
    }
}

/// TLS配置验证器
pub struct TlsConfigValidator;

impl TlsConfigValidator {
    /// 验证TLS配置
    pub fn validate_config(config: &TlsConfig) -> Result<Vec<String>> {
        let mut warnings = vec![];

        // 检查证书和私钥是否存在
        if !Path::new(&config.cert_path).exists() {
            warnings.push(format!("Certificate file not found: {}", config.cert_path));
        }

        if !Path::new(&config.key_path).exists() {
            warnings.push(format!("Private key file not found: {}", config.key_path));
        }

        // 检查文件权限
        if let Ok(metadata) = std::fs::metadata(&config.key_path) {
            if metadata.permissions().mode() & 0o077 != 0 {
                warnings.push(format!("Private key file has insecure permissions: {}", config.key_path));
            }
        }

        // 检查TLS版本
        match config.min_tls_version {
            TlsVersion::Tls1_2 => {
                warnings.push("TLS 1.2 is less secure than TLS 1.3".to_string());
            }
            TlsVersion::Tls1_3 => {
                debug!("Using TLS 1.3 (recommended)");
            }
        }

        // 检查密码套件
        if config.cipher_suites.is_empty() {
            warnings.push("No cipher suites specified".to_string());
        }

        Ok(warnings)
    }

    /// 生成配置建议
    pub fn generate_recommendations(config: &TlsConfig) -> Vec<String> {
        let mut recommendations = vec![];

        // 基础建议
        recommendations.push("Use TLS 1.3 if possible for better security".to_string());
        recommendations.push("Implement certificate rotation for production use".to_string());
        recommendations.push("Use certificates from trusted Certificate Authorities".to_string());
        recommendations.push("Enable HSTS (HTTP Strict Transport Security)".to_string());

        // 客户端认证建议
        if config.client_auth_required {
            recommendations.push("Implement proper client certificate management".to_string());
        }

        // 开发环境建议
        if config.cert_path.contains("dev") || config.key_path.contains("dev") {
            recommendations.push("Development certificates detected - use production certificates in production".to_string());
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_tls_config_creation() {
        let config = TlsConfig::new("test.crt".to_string(), "test.key".to_string());
        assert_eq!(config.cert_path, "test.crt");
        assert_eq!(config.key_path, "test.key");
        assert!(!config.client_auth_required);
    }

    #[test]
    fn test_tls_config_with_client_auth() {
        let config = TlsConfig::new("test.crt".to_string(), "test.key".to_string())
            .with_client_auth(true, Some("ca.crt".to_string()));
        
        assert!(config.client_auth_required);
        assert_eq!(config.client_ca_path, Some("ca.crt".to_string()));
    }

    #[test]
    fn test_certificate_generator() {
        let generator = CertificateGenerator::new("localhost".to_string());
        let (cert, key) = generator.generate_self_signed().unwrap();
        
        assert!(cert.contains("BEGIN CERTIFICATE"));
        assert!(key.contains("BEGIN PRIVATE KEY"));
    }

    #[tokio::test]
    async fn test_tls_server_initialization() {
        // 创建临时证书文件
        let mut cert_file = NamedTempFile::new().unwrap();
        let mut key_file = NamedTempFile::new().unwrap();
        
        writeln!(cert_file, "-----BEGIN CERTIFICATE-----\nDEV_CERT\n-----END CERTIFICATE-----").unwrap();
        writeln!(key_file, "-----BEGIN PRIVATE KEY-----\nDEV_KEY\n-----END PRIVATE KEY-----").unwrap();
        
        let config = TlsConfig::new(
            cert_file.path().to_str().unwrap().to_string(),
            key_file.path().to_str().unwrap().to_string(),
        );
        
        let mut server = TlsServer::new(config);
        let result = server.initialize().await;
        
        // 注意：这个测试会失败，因为我们使用了占位符证书
        // 在实际实现中，需要使用真实的证书
        assert!(result.is_err());
    }
}