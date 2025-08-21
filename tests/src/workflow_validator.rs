use std::fs;
use serde::{Deserialize, Serialize};
use regex::Regex;

/// GitHub Actions 工作流配置验证器
#[derive(Debug)]
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
    pub fn new(_workflow_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
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
        let yaml_value = self.validate_syntax(&mut result);
        
        // 安全性检查
        self.validate_security(&mut result);
        
        // 性能检查
        self.validate_performance(&mut result);
        
        // 最佳实践检查
        self.validate_best_practices(&mut result);

        // 优化实现 - 检查作业依赖关系
        self.validate_job_dependencies(&yaml_value, &mut result);
        
        result.is_valid = result.errors.is_empty();
        result
    }

    /// 验证基础语法
    fn validate_syntax(&self, result: &mut WorkflowValidationResult) -> serde_yaml::Value {
        // 检查YAML格式
        let yaml_value = match serde_yaml::from_str::<serde_yaml::Value>(&self.content) {
            Ok(value) => value,
            Err(e) => {
                result.errors.push(format!("Invalid YAML syntax: {}", e));
                return serde_yaml::Value::Null;
            }
        };

        // 检查必需字段
        let required_fields = vec!["name", "on", "jobs"];
        for field in required_fields {
            if !yaml_value.get(field).is_some() {
                result.errors.push(format!("Missing required field: {}", field));
            }
        }

        // 优化实现 - 检查工作流结构
        if let Some(jobs) = yaml_value.get("jobs") {
            if let Some(jobs_map) = jobs.as_mapping() {
                // 检查空的作业部分
                if jobs_map.is_empty() {
                    result.errors.push("Empty jobs section".to_string());
                }

                // 检查每个作业的配置
                for (job_name, job_config) in jobs_map {
                    if let Some(job_map) = job_config.as_mapping() {
                        // 检查无效的作业配置
                        if !job_map.contains_key("runs-on") && !job_map.contains_key("uses") {
                            result.errors.push(format!("Invalid job configuration for job '{}'", job_name.as_str().unwrap_or("unknown")));
                        }

                        // 检查步骤配置
                        if let Some(steps) = job_map.get("steps") {
                            if let Some(steps_array) = steps.as_sequence() {
                                // 检查空的步骤列表
                                if steps_array.is_empty() {
                                    result.errors.push("Empty steps list".to_string());
                                }

                                // 检查每个步骤的配置
                                for (step_index, step) in steps_array.iter().enumerate() {
                                    if let Some(step_map) = step.as_mapping() {
                                        let has_run = step_map.contains_key("run");
                                        let has_uses = step_map.contains_key("uses");
                                        
                                        // 检查既没有run也没有uses的步骤
                                        if !has_run && !has_uses {
                                            result.errors.push(format!("Invalid step configuration at step {}: missing 'run' or 'uses'", step_index));
                                        }
                                        
                                        // 检查同时使用run和uses的步骤
                                        if has_run && has_uses {
                                            result.errors.push(format!("Invalid step configuration at step {}: cannot use both 'run' and 'uses'", step_index));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 检查触发条件
        if let Some(on_field) = yaml_value.get("on") {
            if let Some(on_map) = on_field.as_mapping() {
                if on_map.is_empty() {
                    result.errors.push("Empty triggers".to_string());
                }

                // 检查cron表达式
                if let Some(schedule) = on_map.get("schedule") {
                    if let Some(schedule_array) = schedule.as_sequence() {
                        for cron_item in schedule_array {
                            if let Some(cron_map) = cron_item.as_mapping() {
                                if let Some(cron_expr) = cron_map.get("cron") {
                                    if let Some(expr_str) = cron_expr.as_str() {
                                        // 简单的cron表达式验证
                                        if !expr_str.contains('*') || expr_str.len() < 5 {
                                            result.errors.push("Invalid cron expression".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 检查事件类型
                for event_key in on_map.keys() {
                    if let Some(event_str) = event_key.as_str() {
                        if !["push", "pull_request", "workflow_dispatch", "schedule", "release", "issues", "discussion"].contains(&event_str) {
                            result.errors.push(format!("Invalid event type: {}", event_str));
                        }
                    }
                }
            }
        }
        
        yaml_value
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

    /// 验证作业依赖关系
    fn validate_job_dependencies(&self, yaml_value: &serde_yaml::Value, result: &mut WorkflowValidationResult) {
        if let Some(jobs) = yaml_value.get("jobs") {
            if let Some(jobs_map) = jobs.as_mapping() {
                let job_names: Vec<String> = jobs_map.keys()
                    .filter_map(|k| k.as_str().map(|s| s.to_string()))
                    .collect();

                for (job_name, job_config) in jobs_map {
                    if let Some(job_map) = job_config.as_mapping() {
                        if let Some(job_name_str) = job_name.as_str() {
                            // 检查自依赖
                            if let Some(needs) = job_map.get("needs") {
                                if let Some(needs_array) = needs.as_sequence() {
                                    for need in needs_array {
                                        if let Some(need_str) = need.as_str() {
                                            if need_str == job_name_str {
                                                result.errors.push(format!("Job '{}' cannot depend on itself", job_name_str));
                                            }
                                        }
                                    }
                                }
                            }

                            // 检查不存在的依赖
                            if let Some(needs) = job_map.get("needs") {
                                if let Some(needs_array) = needs.as_sequence() {
                                    for need in needs_array {
                                        if let Some(need_str) = need.as_str() {
                                            if !job_names.contains(&need_str.to_string()) {
                                                result.errors.push(format!("Job '{}' depends on non-existent job '{}'", job_name_str, need_str));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 检查循环依赖（简单检查）
                for job_name in &job_names {
                    self.check_circular_dependencies(jobs_map, job_name, &mut Vec::new(), result);
                }
            }
        }
    }

    /// 检查循环依赖
    fn check_circular_dependencies(&self, jobs_map: &serde_yaml::Mapping, job_name: &str, path: &mut Vec<String>, result: &mut WorkflowValidationResult) {
        if path.contains(&job_name.to_string()) {
            result.errors.push(format!("Circular dependency detected: {} -> {}", path.join(" -> "), job_name));
            return;
        }

        path.push(job_name.to_string());

        if let Some(job_config) = jobs_map.get(&serde_yaml::Value::String(job_name.to_string())) {
            if let Some(job_map) = job_config.as_mapping() {
                if let Some(needs) = job_map.get("needs") {
                    if let Some(needs_array) = needs.as_sequence() {
                        for need in needs_array {
                            if let Some(need_str) = need.as_str() {
                                self.check_circular_dependencies(jobs_map, need_str, path, result);
                            }
                        }
                    }
                }
            }
        }

        path.pop();
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