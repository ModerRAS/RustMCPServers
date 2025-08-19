use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use regex::Regex;

/// GitHub Actions 工作流配置验证器
pub struct WorkflowValidator {
    pub workflow_path: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl WorkflowValidator {
    /// 创建新的验证器实例
    pub fn new(workflow_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(workflow_path)?;
        Ok(Self {
            workflow_path: workflow_path.to_string(),
            content,
        })
    }

    /// 验证工作流配置
    pub fn validate(&self) -> WorkflowValidationResult {
        let mut result = WorkflowValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        };

        // 基础语法检查
        self.validate_syntax(&mut result);
        
        // 安全性检查
        self.validate_security(&mut result);
        
        // 性能检查
        self.validate_performance(&mut result);
        
        // 最佳实践检查
        self.validate_best_practices(&mut result);

        result.is_valid = result.errors.is_empty();
        result
    }

    /// 验证基础语法
    fn validate_syntax(&self, result: &mut WorkflowValidationResult) {
        // 检查YAML格式
        let yaml_value = match serde_yaml::from_str::<serde_yaml::Value>(&self.content) {
            Ok(value) => value,
            Err(e) => {
                result.errors.push(format!("Invalid YAML syntax: {}", e));
                return;
            }
        };

        // 检查必需字段
        let required_fields = vec!["name", "on", "jobs"];
        for field in required_fields {
            if !yaml_value.get(field).is_some() {
                result.errors.push(format!("Missing required field: {}", field));
            }
        }
    }

    /// 验证安全性
    fn validate_security(&self, result: &mut WorkflowValidationResult) {
        // 检查硬编码密钥
        let secret_patterns = vec![
            r#"api[_-]?key\s*:\s*["'][^"']+["']"#,
            r#"secret\s*:\s*["'][^"']+["']"#,
            r#"password\s*:\s*["'][^"']+["']"#,
        ];

        for pattern in secret_patterns {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&self.content) {
                result.warnings.push("Potential hardcoded secrets detected".to_string());
            }
        }

        // 检查是否使用最新版本的动作
        if self.content.contains("actions/checkout@v2") {
            result.warnings.push("Using outdated checkout action, consider v4".to_string());
        }
    }

    /// 验证性能
    fn validate_performance(&self, result: &mut WorkflowValidationResult) {
        // 检查缓存配置
        if !self.content.contains("actions/cache@") {
            result.warnings.push("No caching configured, consider adding cargo cache".to_string());
        }

        // 检查并行作业
        let job_count = self.content.matches("runs-on:").count();
        if job_count > 3 {
            result.info.push("Multiple jobs detected, ensure they can run in parallel".to_string());
        }
    }

    /// 验证最佳实践
    fn validate_best_practices(&self, result: &mut WorkflowValidationResult) {
        // 检查是否有错误处理
        if !self.content.contains("continue-on-error") && !self.content.contains("if: failure()") {
            result.warnings.push("No error handling found in workflow".to_string());
        }

        // 检查权限设置
        if !self.content.contains("permissions:") {
            result.warnings.push("No permissions specified, consider setting minimal permissions".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_valid_workflow() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: echo "Hello World"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();
        
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();
        
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_missing_required_field() {
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    runs-on: ubuntu-latest
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();
        
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();
        
        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Missing required field: on".to_string()));
    }

    #[test]
    fn test_security_warning() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: echo "api_key: 'secret123'"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();
        
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();
        
        assert!(result.warnings.len() > 0);
    }
}