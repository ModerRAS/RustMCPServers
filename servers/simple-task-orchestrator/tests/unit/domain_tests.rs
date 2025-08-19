#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_task_id_new() {
        let task_id = TaskId::new();
        assert!(!task_id.to_string().is_empty());
    }

    #[test]
    fn test_task_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let task_id = TaskId::from_uuid(uuid);
        assert_eq!(task_id.as_uuid(), &uuid);
    }

    #[test]
    fn test_task_id_display() {
        let task_id = TaskId::new();
        let display = format!("{}", task_id);
        assert_eq!(display, task_id.to_string());
    }

    #[test]
    fn test_task_id_default() {
        let task_id = TaskId::default();
        assert!(!task_id.to_string().is_empty());
    }

    #[test]
    fn test_worker_id_new_valid() {
        let worker_id = WorkerId::new("test_worker".to_string());
        assert!(worker_id.is_ok());
        assert_eq!(worker_id.unwrap().as_str(), "test_worker");
    }

    #[test]
    fn test_worker_id_new_empty() {
        let worker_id = WorkerId::new("".to_string());
        assert!(worker_id.is_err());
    }

    #[test]
    fn test_worker_id_new_too_long() {
        let long_id = "a".repeat(101);
        let worker_id = WorkerId::new(long_id);
        assert!(worker_id.is_err());
    }

    #[test]
    fn test_worker_id_display() {
        let worker_id = WorkerId::new("test".to_string()).unwrap();
        let display = format!("{}", worker_id);
        assert_eq!(display, "test");
    }

    #[test]
    fn test_task_status_is_terminal() {
        assert!(!TaskStatus::Waiting.is_terminal());
        assert!(!TaskStatus::Working.is_terminal());
        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(TaskStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_task_status_can_transition_to() {
        // 有效转换
        assert!(TaskStatus::Waiting.can_transition_to(&TaskStatus::Working));
        assert!(TaskStatus::Waiting.can_transition_to(&TaskStatus::Cancelled));
        assert!(TaskStatus::Working.can_transition_to(&TaskStatus::Completed));
        assert!(TaskStatus::Working.can_transition_to(&TaskStatus::Failed));
        assert!(TaskStatus::Working.can_transition_to(&TaskStatus::Cancelled));
        assert!(TaskStatus::Failed.can_transition_to(&TaskStatus::Waiting));

        // 无效转换
        assert!(!TaskStatus::Waiting.can_transition_to(&TaskStatus::Completed));
        assert!(!TaskStatus::Completed.can_transition_to(&TaskStatus::Working));
        assert!(!TaskStatus::Failed.can_transition_to(&TaskStatus::Completed));
    }

    #[test]
    fn test_task_status_default() {
        assert_eq!(TaskStatus::default(), TaskStatus::Waiting);
    }

    #[test]
    fn test_task_priority_ord() {
        assert!(TaskPriority::High > TaskPriority::Medium);
        assert!(TaskPriority::Medium > TaskPriority::Low);
        assert!(TaskPriority::High > TaskPriority::Low);
    }

    #[test]
    fn test_task_priority_as_i32() {
        assert_eq!(TaskPriority::Low.as_i32(), 1);
        assert_eq!(TaskPriority::Medium.as_i32(), 2);
        assert_eq!(TaskPriority::High.as_i32(), 3);
    }

    #[test]
    fn test_task_priority_default() {
        assert_eq!(TaskPriority::default(), TaskPriority::Medium);
    }

    #[test]
    fn test_task_result_success() {
        let result = TaskResult::success("test output".to_string());
        assert_eq!(result.status, "success");
        assert_eq!(result.output, "test output");
        assert_eq!(result.duration_ms, 0);
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_task_result_failure() {
        let result = TaskResult::failure("test error".to_string());
        assert_eq!(result.status, "failure");
        assert_eq!(result.output, "test error");
        assert_eq!(result.duration_ms, 0);
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_task_new() {
        let task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::High,
            vec!["tag1".to_string(), "tag2".to_string()],
        );

        assert_eq!(task.work_directory, "/test");
        assert_eq!(task.prompt, "test prompt");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.tags, vec!["tag1".to_string(), "tag2".to_string()]);
        assert_eq!(task.status, TaskStatus::Waiting);
        assert!(task.worker_id.is_none());
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
        assert!(task.result.is_none());
        assert!(task.error_message.is_none());
        assert_eq!(task.retry_count, 0);
        assert_eq!(task.max_retries, 3);
    }

    #[test]
    fn test_task_start() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        let worker_id = WorkerId::new("worker1".to_string()).unwrap();
        let result = task.start(worker_id.clone());

        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Working);
        assert_eq!(task.worker_id, Some(worker_id));
        assert!(task.started_at.is_some());
    }

    #[test]
    fn test_task_start_invalid_transition() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        task.status = TaskStatus::Completed;
        let worker_id = WorkerId::new("worker1".to_string()).unwrap();
        let result = task.start(worker_id);

        assert!(result.is_err());
    }

    #[test]
    fn test_task_complete() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        task.status = TaskStatus::Working;
        let result = TaskResult::success("done".to_string());
        let complete_result = task.complete(result.clone());

        assert!(complete_result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.result, Some(result));
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_fail() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        task.status = TaskStatus::Working;
        let fail_result = task.fail("error occurred".to_string());

        assert!(fail_result.is_ok());
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error_message, Some("error occurred".to_string()));
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_cancel() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        let cancel_result = task.cancel(Some("cancelled".to_string()));

        assert!(cancel_result.is_ok());
        assert_eq!(task.status, TaskStatus::Cancelled);
        assert_eq!(task.error_message, Some("cancelled".to_string()));
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_retry() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        task.status = TaskStatus::Failed;
        let retry_result = task.retry();

        assert!(retry_result.is_ok());
        assert_eq!(task.status, TaskStatus::Waiting);
        assert!(task.worker_id.is_none());
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
        assert!(task.error_message.is_none());
        assert_eq!(task.retry_count, 1);
    }

    #[test]
    fn test_task_retry_max_retries() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        task.status = TaskStatus::Failed;
        task.retry_count = 3;
        let retry_result = task.retry();

        assert!(retry_result.is_err());
    }

    #[test]
    fn test_task_retry_only_failed() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        task.status = TaskStatus::Working;
        let retry_result = task.retry();

        assert!(retry_result.is_err());
    }

    #[test]
    fn test_task_is_expired() {
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        // 未开始的任务不应过期
        assert!(!task.is_expired(3600));

        // 刚开始的任务不应过期
        task.status = TaskStatus::Working;
        task.started_at = Some(Utc::now());
        assert!(!task.is_expired(3600));

        // 模拟过期任务
        task.started_at = Some(Utc::now() - chrono::Duration::hours(2));
        assert!(task.is_expired(3600));
    }

    #[test]
    fn test_task_filter_new() {
        let filter = TaskFilter::new();
        assert!(filter.status.is_none());
        assert!(filter.priority.is_none());
        assert!(filter.worker_id.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    #[test]
    fn test_task_filter_builder() {
        let filter = TaskFilter::new()
            .with_status(TaskStatus::Working)
            .with_priority(TaskPriority::High)
            .with_worker_id("worker1".to_string())
            .with_limit(10)
            .with_offset(5);

        assert_eq!(filter.status, Some(TaskStatus::Working));
        assert_eq!(filter.priority, Some(TaskPriority::High));
        assert_eq!(filter.worker_id, Some("worker1".to_string()));
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.offset, Some(5));
    }

    #[test]
    fn test_task_statistics_new() {
        let stats = TaskStatistics::new();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.failed_tasks, 0);
        assert_eq!(stats.cancelled_tasks, 0);
        assert_eq!(stats.active_tasks, 0);
        assert_eq!(stats.success_rate, 0.0);
    }

    #[test]
    fn test_api_response_success() {
        let data = "test data".to_string();
        let response = ApiResponse::success(data.clone());

        assert!(response.success);
        assert_eq!(response.data, Some(data));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let error = ApiError::new("TEST_ERROR".to_string(), "test error".to_string());
        let response = ApiResponse::error(error.clone());

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some(error));
    }

    #[test]
    fn test_api_error_new() {
        let error = ApiError::new("CODE".to_string(), "message".to_string());
        assert_eq!(error.code, "CODE");
        assert_eq!(error.message, "message");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_api_error_with_details() {
        let details = serde_json::json!({"key": "value"});
        let error = ApiError::with_details("CODE".to_string(), "message".to_string(), details.clone());
        assert_eq!(error.code, "CODE");
        assert_eq!(error.message, "message");
        assert_eq!(error.details, Some(details));
    }

    #[test]
    fn test_create_task_request() {
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: "test prompt".to_string(),
            priority: Some(TaskPriority::High),
            tags: Some(vec!["tag1".to_string()]),
        };

        assert_eq!(request.work_directory, "/test");
        assert_eq!(request.prompt, "test prompt");
        assert_eq!(request.priority, Some(TaskPriority::High));
        assert_eq!(request.tags, Some(vec!["tag1".to_string()]));
    }

    #[test]
    fn test_acquire_task_request() {
        let request = AcquireTaskRequest {
            work_path: "/test".to_string(),
            worker_id: "worker1".to_string(),
        };

        assert_eq!(request.work_path, "/test");
        assert_eq!(request.worker_id, "worker1");
    }

    #[test]
    fn test_complete_task_request() {
        let result = TaskResult::success("done".to_string());
        let request = CompleteTaskRequest {
            original_prompt: Some("original".to_string()),
            result: Some(result),
        };

        assert_eq!(request.original_prompt, Some("original".to_string()));
        assert!(request.result.is_some());
    }
}