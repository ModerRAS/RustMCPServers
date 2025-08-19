use std::process::Command;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::domain::{Task, TaskResult};
use anyhow::{Result, Context, anyhow};
use super::TaskExecutor;

/// Claude Code 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    /// Claude Code CLI 路径
    pub claude_path: String,
    /// 使用的模型
    pub model: Option<String>,
    /// 超时时间（秒）
    pub timeout: u64,
    /// 工作目录
    pub work_directory: String,
    /// 是否启用详细输出
    pub verbose: bool,
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        Self {
            claude_path: "claude".to_string(),
            model: Some("claude-sonnet-4-20250514".to_string()),
            timeout: 600,
            work_directory: ".".to_string(),
            verbose: false,
        }
    }
}

/// Claude Code 执行器
pub struct ClaudeCodeExecutor {
    config: ClaudeCodeConfig,
}

impl ClaudeCodeExecutor {
    pub fn new(config: ClaudeCodeConfig) -> Self {
        Self { config }
    }

    pub fn with_work_directory(work_directory: String) -> Self {
        Self {
            config: ClaudeCodeConfig {
                work_directory,
                ..Default::default()
            },
        }
    }

    /// 执行任务
    pub async fn execute_task(&self, task: &Task) -> Result<TaskResult> {
        let start_time = std::time::Instant::now();
        
        // 构建系统提示
        let system_prompt = self.build_system_prompt(task);
        
        // 构建 Claude Code 命令
        let mut command = self.build_claude_command(&system_prompt)?;
        
        // 执行命令
        let output = self.execute_command(command).await?;
        
        // 解析结果
        let result = self.parse_output(output, start_time.elapsed())?;
        
        Ok(result)
    }

    /// 构建系统提示
    fn build_system_prompt(&self, task: &Task) -> String {
        format!(
            r#"你是一个专业的编程助手。请根据以下要求完成任务：

工作目录: {}
任务描述: {}
优先级: {:?}

要求:
1. 仔细分析任务需求
2. 提供详细的解决方案
3. 确保代码质量最佳实践
4. 如果需要修改文件，请提供具体的修改建议
5. 如果只是分析任务，请提供详细的分析报告

请在工作目录 {} 中执行任务。

请用中文回复，并提供详细的工作步骤和结果。"#,
            task.work_directory,
            task.prompt,
            task.priority,
            task.work_directory
        )
    }

    /// 构建 Claude Code 命令
    fn build_claude_command(&self, system_prompt: &str) -> Result<Command> {
        let mut command = Command::new(&self.config.claude_path);
        
        // 基本参数
        command.arg("-p");
        
        // 系统提示（非Windows平台）
        if !cfg!(target_os = "windows") {
            command.arg("--system-prompt").arg(system_prompt);
        }
        
        // 通用参数
        command.args([
            "--verbose",
            "--output-format", "stream-json",
            "--disallowed-tools", "Task,Bash,Glob,Grep,LS,Read,Edit,MultiEdit,Write,NotebookRead,NotebookEdit,WebFetch,TodoRead,TodoWrite,WebSearch",
            "--max-turns", "1"
        ]);
        
        // 模型参数
        if let Some(ref model) = self.config.model {
            command.arg("--model").arg(model);
        }
        
        // 设置工作目录
        command.current_dir(&self.config.work_directory);
        
        Ok(command)
    }

    /// 执行命令
    async fn execute_command(&self, mut command: Command) -> Result<String> {
        // 使用 tokio 运行命令
        let output = tokio::task::spawn_blocking(move || {
            command.output()
        })
        .await
        .context("Failed to spawn Claude Code process")?
        .context("Failed to execute Claude Code command")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Claude Code execution failed: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// 解析输出
    fn parse_output(&self, output: String, duration: Duration) -> Result<TaskResult> {
        // 解析流式 JSON 输出
        let mut response_text = String::new();
        let mut has_error = false;
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(chunk) => {
                    match chunk.get("type").and_then(|t| t.as_str()) {
                        Some("assistant") => {
                            if let Some(message) = chunk.get("message") {
                                if let Some(content) = message.get("content") {
                                    if let Some(content_array) = content.as_array() {
                                        for content_item in content_array {
                                            if let Some(text_block) = content_item.get("text") {
                                                if let Some(text) = text_block.as_str() {
                                                    response_text.push_str(text);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some("result") => {
                            // 处理最终结果
                            if let Some(result_data) = chunk.get("result") {
                                if let Some(error) = result_data.get("error") {
                                    has_error = true;
                                    response_text = format!("Claude Code execution error: {}", error);
                                }
                            }
                        }
                        Some("system") => {
                            // 系统消息，可以忽略或记录
                            if self.config.verbose {
                                eprintln!("System message: {}", chunk);
                            }
                        }
                        _ => {
                            // 其他类型的消息
                            if self.config.verbose {
                                eprintln!("Unknown message type: {}", chunk);
                            }
                        }
                    }
                }
                Err(e) => {
                    if self.config.verbose {
                        eprintln!("Failed to parse JSON line: {} - Error: {}", line, e);
                    }
                    // 继续处理下一行
                }
            }
        }
        
        // 检查是否有响应
        if response_text.is_empty() {
            return Ok(TaskResult::failure("No response from Claude Code".to_string()));
        }
        
        // 创建结果
        let result = if has_error {
            TaskResult::failure(response_text)
        } else {
            TaskResult::success(response_text)
        };
        
        // 设置持续时间
        let result_with_duration = TaskResult {
            duration_ms: duration.as_millis() as u64,
            ..result
        };
        
        Ok(result_with_duration)
    }

    /// 验证 Claude Code 是否可用
    pub async fn validate_claude_code(&self) -> Result<bool> {
        let mut command = Command::new(&self.config.claude_path);
        command.arg("--version");
        
        let output = tokio::task::spawn_blocking(move || {
            command.output()
        })
        .await
        .context("Failed to spawn Claude Code validation process")?;
        
        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// 获取支持的模型列表
    pub async fn get_supported_models(&self) -> Result<Vec<String>> {
        // 这里返回已知的支持模型列表
        Ok(vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-opus-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
        ])
    }
}

#[async_trait::async_trait]
impl TaskExecutor for ClaudeCodeExecutor {
    async fn execute(&self, task: &Task) -> Result<TaskResult> {
        self.execute_task(task).await
    }

    async fn validate(&self) -> Result<bool> {
        self.validate_claude_code().await
    }

    fn name(&self) -> &'static str {
        "claude_code"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_code_executor_creation() {
        let config = ClaudeCodeConfig::default();
        let executor = ClaudeCodeExecutor::new(config);
        
        assert_eq!(executor.config.claude_path, "claude");
        assert!(executor.config.model.is_some());
    }

    #[tokio::test]
    async fn test_build_system_prompt() {
        let task = crate::domain::Task::new(
            "/test".to_string(),
            "Write a hello world function".to_string(),
            crate::domain::TaskPriority::Medium,
            vec!["test".to_string()],
        );
        
        let executor = ClaudeCodeExecutor::with_work_directory("/test".to_string());
        let prompt = executor.build_system_prompt(&task);
        
        assert!(prompt.contains("工作目录: /test"));
        assert!(prompt.contains("任务描述: Write a hello world function"));
        assert!(prompt.contains("请用中文回复"));
    }

    #[tokio::test]
    async fn test_supported_models() {
        let executor = ClaudeCodeExecutor::with_work_directory(".".to_string());
        let models = executor.get_supported_models().await.unwrap();
        
        assert!(!models.is_empty());
        assert!(models.contains(&"claude-sonnet-4-20250514".to_string()));
    }
}