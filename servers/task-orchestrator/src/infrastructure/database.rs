use sqlx::{SqlitePool, Sqlite, Pool, sqlite::SqliteConnectOptions, ConnectOptions};
use sqlx::migrate::MigrateDatabase;
use chrono::{DateTime, Utc};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{Task, TaskId, TaskStatus, TaskPriority, TaskHistory};
use crate::models::{TaskRecord, TaskHistoryRecord, TaskFilter, TaskStatistics, LockRecord, PerformanceMetricRecord};
use crate::errors::{AppError, AppResult};
use crate::config::DatabaseConfig;

/// 任务仓库特征
#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    /// 创建任务
    async fn create_task(&self, task: &Task) -> AppResult<TaskId>;
    
    /// 获取任务
    async fn get_task(&self, task_id: &TaskId) -> AppResult<Option<Task>>;
    
    /// 更新任务
    async fn update_task(&self, task: &Task) -> AppResult<()>;
    
    /// 删除任务
    async fn delete_task(&self, task_id: &TaskId) -> AppResult<()>;
    
    /// 获取下一个待处理任务
    async fn get_next_task(&self, work_directory: &str, worker_id: &str) -> AppResult<Option<Task>>;
    
    /// 查询任务列表
    async fn list_tasks(&self, filter: &TaskFilter) -> AppResult<(Vec<Task>, u64)>;
    
    /// 获取任务统计
    async fn get_statistics(&self) -> AppResult<TaskStatistics>;
    
    /// 创建任务历史
    async fn create_task_history(&self, history: &TaskHistory) -> AppResult<u64>;
    
    /// 获取任务历史
    async fn get_task_history(&self, task_id: &TaskId) -> AppResult<Vec<TaskHistory>>;
    
    /// 清理过期任务
    async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> AppResult<u64>;
    
    /// 重试失败任务
    async fn retry_failed_tasks(&self, max_retries: u32) -> AppResult<u64>;
}

/// 锁管理器特征
#[async_trait::async_trait]
pub trait LockManager: Send + Sync {
    /// 尝试获取锁
    async fn try_acquire(&self, resource_id: &str, owner_id: &str, ttl_seconds: u64) -> AppResult<bool>;
    
    /// 释放锁
    async fn release(&self, resource_id: &str, owner_id: &str) -> AppResult<bool>;
    
    /// 检查锁是否存在
    async fn check_lock(&self, resource_id: &str) -> AppResult<Option<String>>;
    
    /// 清理过期锁
    async fn cleanup_expired_locks(&self) -> AppResult<u64>;
}

/// 锁管理器盒装trait，用于动态分发
pub type DynLockManager = Arc<dyn LockManager>;

/// SQLite任务仓库实现
pub struct SqliteTaskRepository {
    pool: Pool<Sqlite>,
}

impl SqliteTaskRepository {
    /// 创建新的仓库实例
    pub async fn new(config: &DatabaseConfig) -> AppResult<Self> {
        let pool = Self::create_pool(config).await?;
        
        // 运行数据库迁移
        Self::run_migrations(&pool).await?;
        
        Ok(Self { pool })
    }
    
    /// 使用现有的连接池创建仓库实例
    pub async fn with_pool(pool: Pool<Sqlite>) -> AppResult<Self> {
        // 运行数据库迁移
        Self::run_migrations(&pool).await?;
        
        Ok(Self { pool })
    }
    
    /// 创建数据库连接池
    async fn create_pool(config: &DatabaseConfig) -> AppResult<Pool<Sqlite>> {
        let mut options = SqliteConnectOptions::from_str(&config.url)?;
        
        // 配置SQLite选项
        options = options
            .create_if_missing(true)
            .journal_mode(if config.enable_wal_mode {
                sqlx::sqlite::SqliteJournalMode::Wal
            } else {
                sqlx::sqlite::SqliteJournalMode::Delete
            })
            .synchronous(if config.is_development() {
                sqlx::sqlite::SqliteSynchronous::Normal
            } else {
                sqlx::sqlite::SqliteSynchronous::Full
            })
            .busy_timeout(std::time::Duration::from_secs(config.busy_timeout));
        
        // 设置PRAGMA
        if config.enable_foreign_keys {
            options = options.pragma("foreign_keys", "on");
        }
        
        options = options
            .pragma("temp_store", "memory")
            .pragma("mmap_size", config.mmap_size.to_string())
            .pragma("cache_size", config.cache_size.to_string())
            .pragma("page_size", config.page_size.to_string());
        
        // 创建连接池
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.connection_timeout))
            .idle_timeout(std::time::Duration::from_secs(config.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(config.max_lifetime))
            .connect_with(options)
            .await?;
        
        Ok(pool)
    }
    
    /// 运行数据库迁移
    async fn run_migrations(pool: &Pool<Sqlite>) -> AppResult<()> {
        sqlx::migrate!("./migrations").run(pool).await?;
        Ok(())
    }
    
    /// 在事务中执行操作
    async fn execute_in_transaction<F, T>(&self, operation: F) -> AppResult<T>
    where
        F: for<'a> FnOnce(&'a mut sqlx::sqlite::SqliteConnection) -> AppResult<T> + Send,
        T: Send,
    {
        let mut conn = self.pool.begin().await?;
        let result = operation(&mut conn)?;
        conn.commit().await?;
        Ok(result)
    }
}

#[async_trait::async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn create_task(&self, task: &Task) -> AppResult<TaskId> {
        let task_record = TaskRecord::from_domain(task)?;
        
        let result = sqlx::query(
            r#"
            INSERT INTO tasks (task_id, work_directory, prompt, priority, tags, status, 
                              worker_id, created_at, started_at, completed_at, result, 
                              error_message, retry_count, max_retries, metadata, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&task_record.task_id)
        .bind(&task_record.work_directory)
        .bind(&task_record.prompt)
        .bind(&task_record.priority)
        .bind(&task_record.tags)
        .bind(&task_record.status)
        .bind(&task_record.worker_id)
        .bind(&task_record.created_at)
        .bind(&task_record.started_at)
        .bind(&task_record.completed_at)
        .bind(&task_record.result)
        .bind(&task_record.error_message)
        .bind(&task_record.retry_count)
        .bind(&task_record.max_retries)
        .bind(&task_record.metadata)
        .bind(&task_record.version)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::Internal("Failed to create task".to_string()));
        }
        
        Ok(task.id)
    }
    
    async fn get_task(&self, task_id: &TaskId) -> AppResult<Option<Task>> {
        let record = sqlx::query_as::<_, TaskRecord>(
            "SELECT * FROM tasks WHERE task_id = ?"
        )
        .bind(task_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        match record {
            Some(record) => Ok(Some(record.to_domain()?)),
            None => Ok(None),
        }
    }
    
    async fn update_task(&self, task: &Task) -> AppResult<()> {
        let task_record = TaskRecord::from_domain(task)?;
        
        let result = sqlx::query(
            r#"
            UPDATE tasks 
            SET work_directory = ?, prompt = ?, priority = ?, tags = ?, status = ?,
                worker_id = ?, started_at = ?, completed_at = ?, result = ?, 
                error_message = ?, retry_count = ?, max_retries = ?, metadata = ?, 
                version = version + 1, updated_at = CURRENT_TIMESTAMP
            WHERE task_id = ? AND version = ?
            "#,
        )
        .bind(&task_record.work_directory)
        .bind(&task_record.prompt)
        .bind(&task_record.priority)
        .bind(&task_record.tags)
        .bind(&task_record.status)
        .bind(&task_record.worker_id)
        .bind(&task_record.started_at)
        .bind(&task_record.completed_at)
        .bind(&task_record.result)
        .bind(&task_record.error_message)
        .bind(&task_record.retry_count)
        .bind(&task_record.max_retries)
        .bind(&task_record.metadata)
        .bind(&task_record.task_id)
        .bind(task_record.version - 1)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::ConcurrencyConflict);
        }
        
        Ok(())
    }
    
    async fn delete_task(&self, task_id: &TaskId) -> AppResult<()> {
        let result = sqlx::query(
            "DELETE FROM tasks WHERE task_id = ?"
        )
        .bind(task_id.to_string())
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AppError::TaskNotFound(*task_id));
        }
        
        Ok(())
    }
    
    async fn get_next_task(&self, work_directory: &str, worker_id: &str) -> AppResult<Option<Task>> {
        let record = sqlx::query_as::<_, TaskRecord>(
            "SELECT * FROM tasks 
             WHERE work_directory = ? AND status = 'waiting' 
             ORDER BY priority DESC, created_at ASC 
             LIMIT 1"
        )
        .bind(work_directory)
        .fetch_optional(&self.pool)
        .await?;
        
        match record {
            Some(record) => {
                // 使用乐观锁获取任务
                let updated = sqlx::query(
                    "UPDATE tasks SET status = 'working', worker_id = ?, started_at = CURRENT_TIMESTAMP, version = version + 1 WHERE task_id = ? AND status = 'waiting'"
                )
                .bind(worker_id)
                .bind(&record.task_id)
                .execute(&self.pool)
                .await?;
                
                if updated.rows_affected() > 0 {
                    Ok(Some(record.to_domain()?))
                } else {
                    Ok(None) // 任务已被其他进程获取
                }
            }
            None => Ok(None),
        }
    }
    
    async fn list_tasks(&self, filter: &TaskFilter) -> AppResult<(Vec<Task>, u64)> {
        let mut query = String::from("SELECT * FROM tasks WHERE 1=1");
        let mut params = Vec::new();
        
        // 构建WHERE子句
        if let Some(status) = &filter.status {
            query.push_str(" AND status = ?");
            params.push(status.to_string());
        }
        
        if let Some(work_directory) = &filter.work_directory {
            query.push_str(" AND work_directory LIKE ?");
            params.push(format!("%{}%", work_directory));
        }
        
        if let Some(priority) = &filter.priority {
            query.push_str(" AND priority = ?");
            params.push(priority.to_string());
        }
        
        if let Some(worker_id) = &filter.worker_id {
            query.push_str(" AND worker_id = ?");
            params.push(worker_id.clone());
        }
        
        if let Some(created_after) = &filter.created_after {
            query.push_str(" AND created_at >= ?");
            params.push(created_after.to_rfc3339());
        }
        
        if let Some(created_before) = &filter.created_before {
            query.push_str(" AND created_at <= ?");
            params.push(created_before.to_rfc3339());
        }
        
        // 构建ORDER BY子句
        if let Some(sort_by) = &filter.sort_by {
            query.push_str(&format!(" ORDER BY {}", sort_by));
            if let Some(sort_order) = &filter.sort_order {
                query.push(' ');
                query.push_str(sort_order);
            }
        } else {
            query.push_str(" ORDER BY created_at DESC");
        }
        
        // 获取总数
        let count_query = query.replace("SELECT * FROM", "SELECT COUNT(*) FROM");
        let mut count_query_builder = sqlx::query_builder::QueryBuilder::new(&count_query);
        
        for param in &params {
            count_query_builder.push_bind(param.clone());
        }
        
        let count_result = count_query_builder.build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await?;
        
        let total = count_result.0 as u64;
        
        // 添加LIMIT和OFFSET
        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }
        
        // 执行查询
        let mut query_builder = sqlx::query_builder::QueryBuilder::new(&query);
        
        for param in &params {
            query_builder.push_bind(param.clone());
        }
        
        let records = query_builder.build_query_as::<TaskRecord>()
            .fetch_all(&self.pool)
            .await?;
        
        let tasks = records
            .into_iter()
            .map(|r| r.to_domain())
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok((tasks, total))
    }
    
    async fn get_statistics(&self) -> AppResult<TaskStatistics> {
        let stats = sqlx::query_as::<_, StatsRow>(
            "SELECT 
                COUNT(*) as total_tasks,
                SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) as completed_tasks,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_tasks,
                SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END) as cancelled_tasks,
                SUM(CASE WHEN status = 'waiting' THEN 1 ELSE 0 END) as waiting_tasks,
                SUM(CASE WHEN status = 'working' THEN 1 ELSE 0 END) as working_tasks,
                AVG(CASE WHEN completed_at IS NOT NULL AND started_at IS NOT NULL 
                    THEN (julianday(completed_at) - julianday(started_at)) * 86400 ELSE NULL END) as avg_processing_time
            FROM tasks"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let completed_tasks = stats.completed_tasks.unwrap_or(0) as u64;
        let total_tasks = stats.total_tasks as u64;
        let success_rate = if total_tasks > 0 {
            completed_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };
        
        let active_tasks = stats.waiting_tasks.unwrap_or(0) as u64 + stats.working_tasks.unwrap_or(0) as u64;
        
        Ok(TaskStatistics {
            total_tasks,
            completed_tasks,
            failed_tasks: stats.failed_tasks.unwrap_or(0) as u64,
            cancelled_tasks: stats.cancelled_tasks.unwrap_or(0) as u64,
            active_tasks,
            waiting_tasks: stats.waiting_tasks.unwrap_or(0) as u64,
            working_tasks: stats.working_tasks.unwrap_or(0) as u64,
            success_rate,
            avg_processing_time: stats.avg_processing_time.unwrap_or(0.0),
            tasks_per_hour: 0.0, // 需要基于时间窗口计算
        })
    }
    
    async fn create_task_history(&self, history: &TaskHistory) -> AppResult<u64> {
        let history_record = TaskHistoryRecord::from_domain(history)?;
        
        let result = sqlx::query(
            r#"
            INSERT INTO task_history (task_id, status, worker_id, changed_at, details)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&history_record.task_id)
        .bind(&history_record.status)
        .bind(&history_record.worker_id)
        .bind(&history_record.changed_at)
        .bind(&history_record.details)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid() as u64)
    }
    
    async fn get_task_history(&self, task_id: &TaskId) -> AppResult<Vec<TaskHistory>> {
        let records = sqlx::query_as::<_, TaskHistoryRecord>(
            "SELECT * FROM task_history WHERE task_id = ? ORDER BY changed_at DESC"
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        
        records
            .into_iter()
            .map(|r| r.to_domain())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Internal(e.to_string()))
    }
    
    async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM tasks WHERE status IN ('completed', 'failed', 'cancelled') AND completed_at < ?"
        )
        .bind(older_than)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() as u64)
    }
    
    async fn retry_failed_tasks(&self, max_retries: u32) -> AppResult<u64> {
        let result = sqlx::query(
            "UPDATE tasks SET status = 'waiting', worker_id = NULL, started_at = NULL, retry_count = retry_count + 1 WHERE status = 'failed' AND retry_count < ?"
        )
        .bind(max_retries)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() as u64)
    }
}

/// SQLite锁管理器实现
pub struct SqliteLockManager {
    pool: Pool<Sqlite>,
}

impl SqliteLockManager {
    pub async fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    
    pub async fn with_pool(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LockManager for SqliteLockManager {
    async fn try_acquire(&self, resource_id: &str, owner_id: &str, ttl_seconds: u64) -> AppResult<bool> {
        let expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO locks (resource_id, owner_id, expires_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(resource_id)
        .bind(owner_id)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn release(&self, resource_id: &str, owner_id: &str) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM locks WHERE resource_id = ? AND owner_id = ?"
        )
        .bind(resource_id)
        .bind(owner_id)
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    async fn check_lock(&self, resource_id: &str) -> AppResult<Option<String>> {
        let record = sqlx::query_as::<_, LockRecord>(
            "SELECT * FROM locks WHERE resource_id = ? AND expires_at > CURRENT_TIMESTAMP"
        )
        .bind(resource_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(record.map(|r| r.owner_id))
    }
    
    async fn cleanup_expired_locks(&self) -> AppResult<u64> {
        let result = sqlx::query(
            "DELETE FROM locks WHERE expires_at < CURRENT_TIMESTAMP"
        )
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() as u64)
    }
}

/// 统计查询结果行
#[derive(sqlx::FromRow)]
struct StatsRow {
    total_tasks: Option<i64>,
    completed_tasks: Option<i64>,
    failed_tasks: Option<i64>,
    cancelled_tasks: Option<i64>,
    waiting_tasks: Option<i64>,
    working_tasks: Option<i64>,
    avg_processing_time: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    async fn create_test_pool() -> Pool<Sqlite> {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let url = format!("sqlite://{}", db_path.display());
        
        let config = DatabaseConfig {
            url: url.clone(),
            max_connections: 5,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            enable_wal_mode: true,
            enable_foreign_keys: true,
            busy_timeout: 30,
            cache_size: -64000,
            mmap_size: 268435456,
            page_size: 4096,
        };
        
        let pool = SqliteTaskRepository::create_pool(&config).await.unwrap();
        SqliteTaskRepository::run_migrations(&pool).await.unwrap();
        pool
    }
    
    #[tokio::test]
    async fn test_create_and_get_task() {
        let pool = create_test_pool().await;
        let repo = SqliteTaskRepository::new(&DatabaseConfig::default()).await.unwrap();
        
        let task = Task::new(
            crate::domain::WorkDirectory::new("/test".to_string()).unwrap(),
            crate::domain::Prompt::new("Test task".to_string()).unwrap(),
            TaskPriority::Medium,
            vec![],
        );
        
        let task_id = repo.create_task(&task).await.unwrap();
        let retrieved = repo.get_task(&task_id).await.unwrap().unwrap();
        
        assert_eq!(retrieved.id, task.id);
        assert_eq!(retrieved.work_directory.as_str(), "/test");
        assert_eq!(retrieved.prompt.as_str(), "Test task");
    }
    
    #[tokio::test]
    async fn test_task_lifecycle() {
        let pool = create_test_pool().await;
        let repo = SqliteTaskRepository::new(&DatabaseConfig::default()).await.unwrap();
        
        let mut task = Task::new(
            crate::domain::WorkDirectory::new("/test".to_string()).unwrap(),
            crate::domain::Prompt::new("Test task".to_string()).unwrap(),
            TaskPriority::Medium,
            vec![],
        );
        
        let task_id = repo.create_task(&task).await.unwrap();
        
        // 获取任务
        let next_task = repo.get_next_task("/test", "worker-1").await.unwrap().unwrap();
        assert_eq!(next_task.id, task_id);
        
        // 更新任务状态
        task.start(crate::domain::WorkerId::new("worker-1".to_string()).unwrap()).unwrap();
        repo.update_task(&task).await.unwrap();
        
        // 完成任务
        task.complete(crate::domain::TaskResult::success("Done".to_string())).unwrap();
        repo.update_task(&task).await.unwrap();
        
        // 验证任务状态
        let completed = repo.get_task(&task_id).await.unwrap().unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
    }
}