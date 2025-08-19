use std::fs;
use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// GitHub Actions 安全测试器
pub struct SecurityTester {
    pub workflow_path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityTestResult {
    pub test_name: String,
    pub is_secure: bool,
    pub vulnerabilities: Vec<Vulnerability>,
    pub recommendations: Vec<String>,
    pub score: u8, // 0-100 安全评分
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vulnerability {
    pub severity: Severity,
    pub category: String,
    pub description: String,
    pub line_number: Option<usize>,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl SecurityTester {
    /// 创建新的安全测试器
    pub fn new(workflow_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(workflow_path)?;
        Ok(Self {
            workflow_path: workflow_path.to_string(),
            content,
        })
    }

    /// 执行全面的安全测试
    pub fn run_security_tests(&self) -> SecurityTestResult {
        let mut vulnerabilities = Vec::new();
        let mut recommendations = Vec::new();

        // 1. 密钥泄露检测
        self.check_secret_leaks(&mut vulnerabilities, &mut recommendations);

        // 2. 权限检查
        self.check_permissions(&mut vulnerabilities, &mut recommendations);

        // 3. 代码注入检查
        self.check_code_injection(&mut vulnerabilities, &mut recommendations);

        // 4. 依赖安全检查
        self.check_dependency_security(&mut vulnerabilities, &mut recommendations);

        // 5. 不安全操作检查
        self.check_unsafe_operations(&mut vulnerabilities, &mut recommendations);

        // 6. 网络安全检查
        self.check_network_security(&mut vulnerabilities, &mut recommendations);

        // 计算安全评分
        let score = self.calculate_security_score(&vulnerabilities);
        let is_secure = score >= 70;

        SecurityTestResult {
            test_name: format!("Security Test - {}", self.workflow_path),
            is_secure,
            vulnerabilities,
            recommendations,
            score,
        }
    }

    /// 检查密钥泄露
    fn check_secret_leaks(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        let secret_patterns = vec![
            (r#"api[_-]?key\s*[:=]\s*["'][A-Za-z0-9+/]{20,}["']"#, Severity::Critical),
            (r#"secret\s*[:=]\s*["'][A-Za-z0-9+/]{20,}["']"#, Severity::Critical),
            (r#"password\s*[:=]\s*["'][A-Za-z0-9+/]{8,}["']"#, Severity::Critical),
            (r#"token\s*[:=]\s*["'][A-Za-z0-9+/]{20,}["']"#, Severity::High),
            (r#"AWS_ACCESS_KEY_ID\s*[:=]\s*["'][A-Z0-9]{20}["']"#, Severity::Critical),
            (r#"AWS_SECRET_ACCESS_KEY\s*[:=]\s*["'][A-Za-z0-9+/]{40}["']"#, Severity::Critical),
        ];

        for (pattern, severity) in secret_patterns {
            let re = Regex::new(pattern).unwrap();
            for (line_num, line) in self.content.lines().enumerate() {
                if re.is_match(line) {
                    vulnerabilities.push(Vulnerability {
                        severity: severity.clone(),
                        category: "Secret Leak".to_string(),
                        description: format!("Potential hardcoded secret found: {}", line.trim()),
                        line_number: Some(line_num + 1),
                        recommendation: "Use GitHub Secrets or environment variables instead of hardcoded values".to_string(),
                    });
                }
            }
        }

        if !vulnerabilities.iter().any(|v| v.category == "Secret Leak") {
            recommendations.push("✅ No hardcoded secrets detected".to_string());
        }
    }

    /// 检查权限配置
    fn check_permissions(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        if !self.content.contains("permissions:") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Permissions".to_string(),
                description: "No explicit permissions configured".to_string(),
                line_number: None,
                recommendation: "Configure minimal required permissions for security".to_string(),
            });
        } else {
            // 检查是否有过度权限
            if self.content.contains("contents: write") || self.content.contains("packages: write") {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::Low,
                    category: "Permissions".to_string(),
                    description: "Potentially excessive write permissions detected".to_string(),
                    line_number: None,
                    recommendation: "Review and minimize write permissions".to_string(),
                });
            }
        }

        // 检查GITHUB_TOKEN权限
        if self.content.contains("GITHUB_TOKEN") && !self.content.contains("permissions:") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Permissions".to_string(),
                description: "GITHUB_TOKEN used without explicit permissions".to_string(),
                line_number: None,
                recommendation: "Configure explicit permissions for GITHUB_TOKEN".to_string(),
            });
        }
    }

    /// 检查代码注入风险
    fn check_code_injection(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查shell命令中的用户输入
        let injection_patterns = vec![
            (r"run:\s*\n\s*-\s*.*\$\{\{.*github\.event\..*\}\}", Severity::High),
            (r"run:\s*\n\s*-\s*.*\$\{\{.*inputs\..*\}\}", Severity::High),
            (r"run:\s*\n\s*-\s*.*\$\{\{.*matrix\..*\}\}", Severity::Medium),
        ];

        for (pattern, severity) in injection_patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&self.content) {
                vulnerabilities.push(Vulnerability {
                    severity: severity.clone(),
                    category: "Code Injection".to_string(),
                    description: "Potential code injection vulnerability detected".to_string(),
                    line_number: None,
                    recommendation: "Validate and sanitize user inputs before using in shell commands".to_string(),
                });
            }
        }
    }

    /// 检查依赖安全
    fn check_dependency_security(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查使用的不安全或过时的动作版本
        let outdated_actions = vec![
            ("actions/checkout@v1", Severity::High),
            ("actions/checkout@v2", Severity::Medium),
            ("actions/setup-python@v1", Severity::Medium),
            ("actions/setup-node@v1", Severity::Medium),
            ("actions/cache@v1", Severity::Medium),
        ];

        for (action, severity) in outdated_actions {
            if self.content.contains(action) {
                vulnerabilities.push(Vulnerability {
                    severity: severity.clone(),
                    category: "Dependency Security".to_string(),
                    description: format!("Outdated action version detected: {}", action),
                    line_number: None,
                    recommendation: format!("Update to latest version of {}", action),
                });
            }
        }

        // 检查第三方动作
        let third_party_re = Regex::new(r"uses:\s*[^/]+/[^@]+@").unwrap();
        if third_party_re.is_match(&self.content) {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Low,
                category: "Dependency Security".to_string(),
                description: "Third-party actions detected".to_string(),
                line_number: None,
                recommendation: "Review third-party actions for security and pin to specific commit hash".to_string(),
            });
        }
    }

    /// 检查不安全操作
    fn check_unsafe_operations(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查是否禁用安全功能
        if self.content.contains("continue-on-error: true") {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Low,
                category: "Unsafe Operations".to_string(),
                description: "continue-on-error enabled, may hide security issues".to_string(),
                line_number: None,
                recommendation: "Use continue-on-error sparingly and monitor failures".to_string(),
            });
        }

        // 检查是否在PR中写入操作
        if self.content.contains("pull_request") && 
           (self.content.contains("contents: write") || self.content.contains("packages: write")) {
            vulnerabilities.push(Vulnerability {
                severity: Severity::Medium,
                category: "Unsafe Operations".to_string(),
                description: "Write permissions enabled in pull_request context".to_string(),
                line_number: None,
                recommendation: "Avoid write permissions in pull_request context for security".to_string(),
            });
        }
    }

    /// 检查网络安全
    fn check_network_security(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查是否使用HTTPS
        let http_re = Regex::new(r"http://[^\s\"]+").unwrap();
        for (line_num, line) in self.content.lines().enumerate() {
            if http_re.is_match(line) && !line.contains("localhost") {
                vulnerabilities.push(Vulnerability {
                    severity: Severity::Medium,
                    category: "Network Security".to_string(),
                    description: "Insecure HTTP URL detected".to_string(),
                    line_number: Some(line_num + 1),
                    recommendation: "Use HTTPS instead of HTTP for network requests".to_string(),
                });
            }
        }
    }

    /// 计算安全评分
    fn calculate_security_score(&self, vulnerabilities: &[Vulnerability]) -> u8 {
        let mut score = 100;

        for vuln in vulnerabilities {
            match vuln.severity {
                Severity::Critical => score -= 20,
                Severity::High => score -= 15,
                Severity::Medium => score -= 10,
                Severity::Low => score -= 5,
                Severity::Info => score -= 2,
            }
        }

        score.max(0).min(100)
    }

    /// 生成安全报告
    pub fn generate_security_report(&self, result: &SecurityTestResult) -> String {
        let mut report = String::new();
        report.push_str(&format!("# Security Test Report\n\n"));
        report.push_str(&format!("**Workflow:** {}\n", self.workflow_path));
        report.push_str(&format!("**Security Score:** {}/100\n", result.score));
        report.push_str(&format!("**Status:** {}\n\n", if result.is_secure { "✅ Secure" } else { "❌ Insecure" }));

        if !result.vulnerabilities.is_empty() {
            report.push_str("## Vulnerabilities Found\n\n");
            
            let by_severity = vec![
                (Severity::Critical, "Critical"),
                (Severity::High, "High"),
                (Severity::Medium, "Medium"),
                (Severity::Low, "Low"),
                (Severity::Info, "Info"),
            ];

            for (severity, label) in by_severity {
                let vulns: Vec<_> = result.vulnerabilities.iter()
                    .filter(|v| v.severity == severity)
                    .collect();
                
                if !vulns.is_empty() {
                    report.push_str(&format!("### {}\n\n", label));
                    for vuln in vulns {
                        report.push_str(&format!("- **{}:** {}\n", vuln.category, vuln.description));
                        if let Some(line) = vuln.line_number {
                            report.push_str(&format!("  - Line: {}\n", line));
                        }
                        report.push_str(&format!("  - Recommendation: {}\n\n", vuln.recommendation));
                    }
                }
            }
        }

        if !result.recommendations.is_empty() {
            report.push_str("## Recommendations\n\n");
            for rec in &result.recommendations {
                report.push_str(&format!("- {}\n", rec));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_secret_detection() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: |
          echo "api_key = 'sk-1234567890abcdef'"
          echo "secret = 'super_secret_key_123'"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();

        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();

        assert!(!result.is_secure);
        assert!(result.vulnerabilities.iter().any(|v| v.category == "Secret Leak"));
    }

    #[test]
    fn test_permissions_check() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: echo "test"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();

        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();

        assert!(result.vulnerabilities.iter().any(|v| v.category == "Permissions"));
    }

    #[test]
    fn test_outdated_actions() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v1
      - run: echo "test"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();

        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();

        assert!(result.vulnerabilities.iter().any(|v| v.category == "Dependency Security"));
    }
}