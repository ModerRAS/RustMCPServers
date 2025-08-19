#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use crate::services::*;
    use crate::infrastructure::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };
    use mockall::mock;
    use std::sync::Arc;
    use tower::ServiceExt;
    use uuid::Uuid;

    mock! {
        pub TaskService {}
        impl TaskService for TaskService {
            fn new(
                task_repository: Arc<dyn TaskRepository>,
                lock_manager: Arc<dyn LockManager>,
                max_retries: u32,
                task_timeout: u64,
            ) -> Self;
            
            async fn create_task(&self, request: CreateTaskRequest) -> Result<Task, String>;
            async fn get_task(&self, task_id: &TaskId) -> Result<Task, String>;
            async fn acquire_task(&self, request: AcquireTaskRequest) -> Result<Option<Task>, String>;
            async fn complete_task(&self, task_id: &TaskId, request: CompleteTaskRequest) -> Result<Task, String>;
            async fn fail_task(&self, task_id: &TaskId, error: String) -> Result<Task, String>;
            async fn cancel_task(&self, task_id: &TaskId, reason: Option<String>) -> Result<Task, String>;
            async fn retry_task(&self, task_id: &TaskId) -> Result<Task, String>;
            async fn list_tasks(&self, filter: TaskFilter) -> Result<(Vec<Task>, u64), String>;
            async fn get_statistics(&self) -> Result<TaskStatistics, String>;
            async fn cleanup_expired_tasks(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<u64, String>;
            async fn retry_failed_tasks(&self) -> Result<u64, String>;
            async fn check_task_timeout(&self, task_id: &TaskId) -> Result<bool, String>;
            async fn handle_timeout_tasks(&self) -> Result<u64, String>;
        }
    }

    fn create_mock_task_service() -> MockTaskService {
        MockTaskService::new()
    }

    fn create_test_task() -> Task {
        Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec!["test".to_string()],
        )
    }

    #[tokio::test]
    async fn test_create_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_create_task()
            .returning(move |_| Ok(expected_task.clone()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: "test prompt".to_string(),
            priority: Some(TaskPriority::Medium),
            tags: Some(vec!["test".to_string()]),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_create_task_validation_error() {
        let mut mock_service = create_mock_task_service();
        
        mock_service.expect_create_task()
            .returning(|_| Err("Invalid work directory".to_string()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let request = CreateTaskRequest {
            work_directory: "invalid".to_string(),
            prompt: "test prompt".to_string(),
            priority: None,
            tags: None,
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn test_get_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_get_task()
            .returning(move |_| Ok(expected_task.clone()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/tasks/{}", task_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_get_task_invalid_id() {
        let mock_service = create_mock_task_service();
        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks/invalid-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "INVALID_ID");
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let mut mock_service = create_mock_task_service();
        
        mock_service.expect_get_task()
            .returning(|_| Err("Task not found".to_string()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/v1/tasks/{}", task_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "NOT_FOUND");
    }

    #[tokio::test]
    async fn test_get_next_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_acquire_task()
            .returning(move |_| Ok(Some(expected_task.clone())));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks/next?work_path=/test&worker_id=worker1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_get_next_task_no_task() {
        let mut mock_service = create_mock_task_service();
        
        mock_service.expect_acquire_task()
            .returning(|_| Ok(None));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks/next?work_path=/test&worker_id=worker1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_none());
    }

    #[tokio::test]
    async fn test_complete_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_complete_task()
            .returning(move |_, _| Ok(expected_task.clone()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let request = CompleteTaskRequest {
            original_prompt: None,
            result: Some(TaskResult::success("done".to_string())),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/tasks/{}/complete", task_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_complete_task_invalid_id() {
        let mock_service = create_mock_task_service();
        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let request = CompleteTaskRequest {
            original_prompt: None,
            result: None,
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/invalid-uuid/complete")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "INVALID_ID");
    }

    #[tokio::test]
    async fn test_cancel_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_cancel_task()
            .returning(move |_, _| Ok(expected_task.clone()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let reason_data = serde_json::json!({"reason": "test cancellation"});

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/tasks/{}/cancel", task_id))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&reason_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_retry_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_retry_task()
            .returning(move |_| Ok(expected_task.clone()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/tasks/{}/retry", task_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Task> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_delete_task_success() {
        let mut mock_service = create_mock_task_service();
        let expected_task = create_test_task();
        
        mock_service.expect_get_task()
            .returning(move |_| Ok(expected_task.clone()));
        mock_service.expect_cancel_task()
            .returning(|_, _| Ok(create_test_task()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/tasks/{}", task_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<()> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
    }

    #[tokio::test]
    async fn test_delete_task_not_found() {
        let mut mock_service = create_mock_task_service();
        
        mock_service.expect_get_task()
            .returning(|_| Err("Task not found".to_string()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let task_id = Uuid::new_v4();
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/api/v1/tasks/{}", task_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<()> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "NOT_FOUND");
    }

    #[tokio::test]
    async fn test_list_tasks_success() {
        let mut mock_service = create_mock_task_service();
        let expected_tasks = vec![create_test_task()];
        
        mock_service.expect_list_tasks()
            .returning(move |_| Ok((expected_tasks.clone(), 1)));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Vec<Task>> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
        assert_eq!(response_body.data.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_list_tasks_with_filters() {
        let mut mock_service = create_mock_task_service();
        let expected_tasks = vec![create_test_task()];
        
        mock_service.expect_list_tasks()
            .returning(move |_| Ok((expected_tasks.clone(), 1)));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks?status=waiting&priority=high&limit=10&offset=0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Vec<Task>> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
    }

    #[tokio::test]
    async fn test_list_tasks_invalid_status() {
        let mock_service = create_mock_task_service();
        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks?status=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Vec<Task>> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "INVALID_STATUS");
    }

    #[tokio::test]
    async fn test_list_tasks_invalid_priority() {
        let mock_service = create_mock_task_service();
        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/tasks?priority=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<Vec<Task>> = serde_json::from_slice(&body).unwrap();
        
        assert!(!response_body.success);
        assert!(response_body.error.is_some());
        assert_eq!(response_body.error.unwrap().code, "INVALID_PRIORITY");
    }

    #[tokio::test]
    async fn test_get_statistics_success() {
        let mut mock_service = create_mock_task_service();
        let expected_stats = TaskStatistics {
            total_tasks: 10,
            completed_tasks: 7,
            failed_tasks: 2,
            cancelled_tasks: 1,
            active_tasks: 0,
            success_rate: 0.7,
        };
        
        mock_service.expect_get_statistics()
            .returning(move |_| Ok(expected_stats.clone()));

        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/statistics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<TaskStatistics> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
        let stats = response_body.data.unwrap();
        assert_eq!(stats.total_tasks, 10);
        assert_eq!(stats.success_rate, 0.7);
    }

    #[tokio::test]
    async fn test_health_check() {
        let mock_service = create_mock_task_service();
        let state = ApiState {
            task_service: Arc::new(mock_service),
        };

        let app = create_routes().with_state(state);
        
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let response_body: ApiResponse<serde_json::Value> = serde_json::from_slice(&body).unwrap();
        
        assert!(response_body.success);
        assert!(response_body.data.is_some());
        
        let health_data = response_body.data.unwrap();
        assert_eq!(health_data["status"], "healthy");
        assert!(health_data["timestamp"].is_string());
        assert_eq!(health_data["version"], "1.0.0");
    }
}