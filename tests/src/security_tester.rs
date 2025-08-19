use std::collections::HashMap;
use std::fs;
use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};

/// 安全漏洞严重程度
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// 安全漏洞信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub category: String,
    pub location: String,
    pub recommendation: String,
    pub detected_at: DateTime<Utc>,
}

/// 安全测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTestResult {
    pub workflow_path: String,
    pub vulnerabilities: Vec<Vulnerability>,
    pub recommendations: Vec<String>,
    pub score: u8,
    pub is_secure: bool,
    pub scan_duration: std::time::Duration,
    pub metadata: HashMap<String, Value>,
}

/// 安全测试器
pub struct SecurityTester {
    pub workflow_path: String,
    pub content: String,
    pub scan_timestamp: DateTime<Utc>,
}

impl SecurityTester {
    /// 创建新的安全测试器
    pub fn new(workflow_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(workflow_path);
        let content = fs::read_to_string(path)?;
        
        Ok(Self {
            workflow_path: workflow_path.to_string(),
            content,
            scan_timestamp: Utc::now(),
        })
    }

    /// 运行安全测试
    pub fn run_security_tests(&self) -> SecurityTestResult {
        let start_time = std::time::Instant::now();
        let mut vulnerabilities = Vec::new();
        let mut recommendations = Vec::new();

        // 检查密钥泄露
        self.check_secret_leaks(&mut vulnerabilities, &mut recommendations);
        
        // 检查权限配置
        self.check_permission_config(&mut vulnerabilities, &mut recommendations);
        
        // 检查依赖安全
        self.check_dependency_security(&mut vulnerabilities, &mut recommendations);
        
        // 检查代码注入
        self.check_code_injection(&mut vulnerabilities, &mut recommendations);
        
        // 检查网络安全
        self.check_network_security(&mut vulnerabilities, &mut recommendations);
        
        // 检查不安全操作
        self.check_unsafe_operations(&mut vulnerabilities, &mut recommendations);
        
        // 检查过时组件
        self.check_outdated_components(&mut vulnerabilities, &mut recommendations);

        // 计算安全评分
        let score = self.calculate_security_score(&vulnerabilities);
        let is_secure = score >= 80;

        // 创建元数据
        let mut metadata = HashMap::new();
        metadata.insert("scan_timestamp".to_string(), Value::String(self.scan_timestamp.to_rfc3339()));
        metadata.insert("total_vulnerabilities".to_string(), Value::Number(serde_json::Number::from(vulnerabilities.len())));
        
        let severity_counts = self.count_vulnerabilities_by_severity(&vulnerabilities);
        for (severity, count) in severity_counts {
            metadata.insert(format!("severity_{}", severity.to_string().to_lowercase()), Value::Number(serde_json::Number::from(count)));
        }

        SecurityTestResult {
            workflow_path: self.workflow_path.clone(),
            vulnerabilities,
            recommendations,
            score,
            is_secure,
            scan_duration: start_time.elapsed(),
            metadata,
        }
    }

    /// 检查密钥泄露
    fn check_secret_leaks(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        let secret_patterns = vec![
            (r#"api[_-]?key\s*[:=]\s*["'][A-Za-z0-9+/=_-]{20,}["']"#, Severity::Critical),
            (r#"secret\s*[:=]\s*["'][A-Za-z0-9+/=_-]{20,}["']"#, Severity::Critical),
            (r#"password\s*[:=]\s*["'][A-Za-z0-9+/=_-]{8,}["']"#, Severity::Critical),
            (r#"token\s*[:=]\s*["'][A-Za-z0-9+/=_-]{20,}["']"#, Severity::High),
            (r#"AWS_ACCESS_KEY_ID\s*[:=]\s*["'][A-Z0-9]{20}["']"#, Severity::Critical),
            (r#"AWS_SECRET_ACCESS_KEY\s*[:=]\s*["'][A-Za-z0-9+/=_-]{40}["']"#, Severity::Critical),
        ];

        for (pattern, severity) in secret_patterns {
            let re = Regex::new(pattern).unwrap();
            for (line_num, line) in self.content.lines().enumerate() {
                if re.is_match(line) {
                    vulnerabilities.push(Vulnerability {
                        id: format!("secret_leak_{}", line_num),
                        title: "Hardcoded Secret Detected".to_string(),
                        description: format!("Potential hardcoded secret found in workflow file"),
                        severity: severity.clone(),
                        category: "Secret Leak".to_string(),
                        location: format!("Line {}: {}", line_num + 1, line.trim()),
                        recommendation: "Use GitHub Secrets or environment variables instead of hardcoded values".to_string(),
                        detected_at: self.scan_timestamp,
                    });
                }
            }
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Remove hardcoded secrets and use GitHub Secrets instead".to_string());
        }
    }

    /// 检查权限配置
    fn check_permission_config(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查过宽的权限
        if self.content.contains("contents: write") && self.content.contains("pull_request: write") {
            vulnerabilities.push(Vulnerability {
                id: "broad_permissions".to_string(),
                title: "Overly Broad Permissions".to_string(),
                description: "Workflow has broad permissions that may not be necessary".to_string(),
                severity: Severity::Medium,
                category: "Permission Configuration".to_string(),
                location: "permissions section".to_string(),
                recommendation: "Use principle of least privilege - only grant necessary permissions".to_string(),
                detected_at: self.scan_timestamp,
            });
        }

        // 检查是否设置了权限
        if !self.content.contains("permissions:") {
            vulnerabilities.push(Vulnerability {
                id: "missing_permissions".to_string(),
                title: "Missing Permissions Configuration".to_string(),
                description: "Workflow does not explicitly define permissions".to_string(),
                severity: Severity::Medium,
                category: "Permission Configuration".to_string(),
                location: "workflow file".to_string(),
                recommendation: "Explicitly set permissions to follow principle of least privilege".to_string(),
                detected_at: self.scan_timestamp,
            });
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Review and minimize workflow permissions using principle of least privilege".to_string());
        }
    }

    /// 检查依赖安全
    fn check_dependency_security(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查过时的Actions版本
        let outdated_actions = vec![
            (r#"actions/checkout@v[12]"#, Severity::Medium),
            (r#"actions/setup-python@v[12]"#, Severity::Medium),
            (r#"actions/setup-node@v[12]"#, Severity::Medium),
        ];

        for (pattern, severity) in outdated_actions {
            let re = Regex::new(pattern).unwrap();
            for (line_num, line) in self.content.lines().enumerate() {
                if re.is_match(line) {
                    vulnerabilities.push(Vulnerability {
                        id: format!("outdated_action_{}", line_num),
                        title: "Outdated GitHub Action Version".to_string(),
                        description: format!("Using outdated version of GitHub Action"),
                        severity: severity.clone(),
                        category: "Dependency Security".to_string(),
                        location: format!("Line {}: {}", line_num + 1, line.trim()),
                        recommendation: "Update to latest stable version of the Action".to_string(),
                        detected_at: self.scan_timestamp,
                    });
                }
            }
        }

        // 检查使用特定的commit hash而不是版本标签
        let re = Regex::new(r#"@[a-f0-9]{40}"#).unwrap();
        for (line_num, line) in self.content.lines().enumerate() {
            if re.is_match(line) {
                vulnerabilities.push(Vulnerability {
                    id: format!("pinned_commit_{}", line_num),
                    title: "Action Pinned to Commit Hash".to_string(),
                    description: "Action is pinned to specific commit hash instead of version tag".to_string(),
                    severity: Severity::Low,
                    category: "Dependency Security".to_string(),
                    location: format!("Line {}: {}", line_num + 1, line.trim()),
                    recommendation: "Use version tags instead of commit hashes for better maintainability".to_string(),
                    detected_at: self.scan_timestamp,
                });
            }
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Update GitHub Actions to latest stable versions".to_string());
        }
    }

    /// 检查代码注入
    fn check_code_injection(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查直接使用用户输入的命令
        let injection_patterns = vec![
            (r#"run: \$\{\{\s*github\.event\.comment\.body\s*\}\}"#, Severity::High),
            (r#"run: \$\{\{\s*github\.event\.issue\.title\s*\}\}"#, Severity::High),
            (r#"run: \$\{\{\s*github\.event\.pull_request\.title\s*\}\}"#, Severity::High),
        ];

        for (pattern, severity) in injection_patterns {
            let re = Regex::new(pattern).unwrap();
            for (line_num, line) in self.content.lines().enumerate() {
                if re.is_match(line) {
                    vulnerabilities.push(Vulnerability {
                        id: format!("code_injection_{}", line_num),
                        title: "Potential Code Injection".to_string(),
                        description: "Direct use of user input in shell commands".to_string(),
                        severity: severity.clone(),
                        category: "Code Injection".to_string(),
                        location: format!("Line {}: {}", line_num + 1, line.trim()),
                        recommendation: "Validate and sanitize user input before use in shell commands".to_string(),
                        detected_at: self.scan_timestamp,
                    });
                }
            }
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Validate and sanitize all user inputs used in shell commands".to_string());
        }
    }

    /// 检查网络安全
    fn check_network_security(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查HTTP URL使用
        let re = Regex::new(r#"http://[^"\s]+"#).unwrap();
        for (line_num, line) in self.content.lines().enumerate() {
            if re.is_match(line) {
                vulnerabilities.push(Vulnerability {
                    id: format!("insecure_http_{}", line_num),
                    title: "Insecure HTTP URL".to_string(),
                    description: "Usage of insecure HTTP URL detected".to_string(),
                    severity: Severity::Medium,
                    category: "Network Security".to_string(),
                    location: format!("Line {}: {}", line_num + 1, line.trim()),
                    recommendation: "Use HTTPS instead of HTTP for network requests".to_string(),
                    detected_at: self.scan_timestamp,
                });
            }
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Use HTTPS URLs for all external network requests".to_string());
        }
    }

    /// 检查不安全操作
    fn check_unsafe_operations(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查shell脚本中的不安全操作
        let unsafe_patterns = vec![
            (r#"rm -rf /"#, Severity::Critical),
            (r#"chmod 777"#, Severity::High),
            (r#"sudo"#, Severity::Medium),
            (r#"eval\s*\$"#, Severity::High),
        ];

        for (pattern, severity) in unsafe_patterns {
            let re = Regex::new(pattern).unwrap();
            for (line_num, line) in self.content.lines().enumerate() {
                if re.is_match(line) {
                    vulnerabilities.push(Vulnerability {
                        id: format!("unsafe_operation_{}", line_num),
                        title: "Unsafe Operation Detected".to_string(),
                        description: format!("Potentially unsafe operation found in workflow"),
                        severity: severity.clone(),
                        category: "Unsafe Operations".to_string(),
                        location: format!("Line {}: {}", line_num + 1, line.trim()),
                        recommendation: "Review and replace unsafe operations with safer alternatives".to_string(),
                        detected_at: self.scan_timestamp,
                    });
                }
            }
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Review and replace unsafe operations with safer alternatives".to_string());
        }
    }

    /// 检查过时组件
    fn check_outdated_components(&self, vulnerabilities: &mut Vec<Vulnerability>, recommendations: &mut Vec<String>) {
        // 检查Node.js版本
        let re = Regex::new(r#"node-version:\s*['"]?([0-9]+)\.([0-9]+)\.([0-9]+)['"]?"#).unwrap();
        for (line_num, line) in self.content.lines().enumerate() {
            if let Some(caps) = re.captures(line) {
                let major: u32 = caps[1].parse().unwrap_or(0);
                let minor: u32 = caps[2].parse().unwrap_or(0);
                
                if major < 16 || (major == 16 && minor < 14) {
                    vulnerabilities.push(Vulnerability {
                        id: format!("outdated_nodejs_{}", line_num),
                        title: "Outdated Node.js Version".to_string(),
                        description: "Using outdated Node.js version that may have security vulnerabilities".to_string(),
                        severity: Severity::Medium,
                        category: "Outdated Components".to_string(),
                        location: format!("Line {}: {}", line_num + 1, line.trim()),
                        recommendation: "Update to Node.js 16.14.0 or later".to_string(),
                        detected_at: self.scan_timestamp,
                    });
                }
            }
        }

        if !vulnerabilities.is_empty() {
            recommendations.push("Update runtime versions to latest stable versions with security patches".to_string());
        }
    }

    /// 计算安全评分
    fn calculate_security_score(&self, vulnerabilities: &Vec<Vulnerability>) -> u8 {
        let mut score = 100;
        
        for vuln in vulnerabilities {
            match vuln.severity {
                Severity::Critical => score -= 20,
                Severity::High => score -= 10,
                Severity::Medium => score -= 5,
                Severity::Low => score -= 2,
                Severity::Info => score -= 1,
            }
        }
        
        score.max(0)
    }

    /// 按严重程度统计漏洞
    fn count_vulnerabilities_by_severity(&self, vulnerabilities: &Vec<Vulnerability>) -> HashMap<String, u32> {
        let mut counts = HashMap::new();
        counts.insert("critical".to_string(), 0);
        counts.insert("high".to_string(), 0);
        counts.insert("medium".to_string(), 0);
        counts.insert("low".to_string(), 0);
        counts.insert("info".to_string(), 0);
        
        for vuln in vulnerabilities {
            let key = match vuln.severity {
                Severity::Critical => "critical",
                Severity::High => "high",
                Severity::Medium => "medium",
                Severity::Low => "low",
                Severity::Info => "info",
            };
            *counts.entry(key.to_string()).or_insert(0) += 1;
        }
        
        counts
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
            
            let by_severity = |a: &Vulnerability, b: &Vulnerability| b.severity.cmp(&a.severity);
            let mut sorted_vulns = result.vulnerabilities.clone();
            sorted_vulns.sort_by(by_severity);
            
            for vuln in sorted_vulns {
                report.push_str(&format!("### {} - {}\n\n", vuln.severity.to_string(), vuln.title));
                report.push_str(&format!("**Description:** {}\n", vuln.description));
                report.push_str(&format!("**Category:** {}\n", vuln.category));
                report.push_str(&format!("**Location:** {}\n", vuln.location));
                report.push_str(&format!("**Recommendation:** {}\n\n", vuln.recommendation));
            }
        } else {
            report.push_str("## ✅ No Vulnerabilities Found\n\n");
        }

        if !result.recommendations.is_empty() {
            report.push_str("## Recommendations\n\n");
            for rec in &result.recommendations {
                report.push_str(&format!("- {}\n", rec));
            }
        }

        report.push_str("\n## Scan Details\n\n");
        report.push_str(&format!("- **Scan Duration:** {:?}\n", result.scan_duration));
        report.push_str(&format!("- **Scan Timestamp:** {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        report
    }
}

impl ToString for Severity {
    fn to_string(&self) -> String {
        match self {
            Severity::Critical => "Critical".to_string(),
            Severity::High => "High".to_string(),
            Severity::Medium => "Medium".to_string(),
            Severity::Low => "Low".to_string(),
            Severity::Info => "Info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_security_tester_creation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name: Test Workflow\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(tester.workflow_path, temp_file.path().to_str().unwrap());
    }

    #[test]
    fn test_secret_detection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "api_key: 'sk-1234567890abcdef1234567890abcdef'").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();
        
        assert!(!result.vulnerabilities.is_empty());
        assert!(result.vulnerabilities.iter().any(|v| v.category == "Secret Leak"));
    }

    #[test]
    fn test_permission_check() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "permissions:\n  contents: write\n  pull_request: write").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();
        
        assert!(!result.vulnerabilities.is_empty());
        assert!(result.vulnerabilities.iter().any(|v| v.category == "Permission Configuration"));
    }

    #[test]
    fn test_outdated_actions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "steps:\n  - uses: actions/checkout@v1").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();
        
        assert!(!result.vulnerabilities.is_empty());
        assert!(result.vulnerabilities.iter().any(|v| v.category == "Dependency Security"));
    }

    #[test]
    fn test_code_injection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "run: ${{{{{}}}}}", "github.event.comment.body").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();
        
        assert!(!result.vulnerabilities.is_empty());
        assert!(result.vulnerabilities.iter().any(|v| v.category == "Code Injection"));
    }

    #[test]
    fn test_secure_workflow() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name: Secure Workflow\non: [push]\npermissions:\n  contents: read\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - run: echo 'Hello World'").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();
        
        assert!(result.is_secure);
        assert!(result.score >= 80);
    }

    #[test]
    fn test_report_generation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name: Test\nenv:\n  api_key: 'sk-1234567890abcdef1234567890abcdef'").unwrap();
        
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();
        let report = tester.generate_security_report(&result);
        
        assert!(report.contains("Security Test Report"));
        assert!(report.contains("Security Score:"));
        assert!(report.contains("Secret Leak"));
    }
}