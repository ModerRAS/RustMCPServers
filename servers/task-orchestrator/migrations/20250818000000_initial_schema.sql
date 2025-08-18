-- 任务表
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT UNIQUE NOT NULL,
    work_directory TEXT NOT NULL,
    prompt TEXT NOT NULL,
    priority TEXT DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
    tags TEXT DEFAULT '[]',
    status TEXT DEFAULT 'waiting' CHECK (status IN ('waiting', 'working', 'completed', 'failed', 'cancelled')),
    worker_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at DATETIME,
    completed_at DATETIME,
    result TEXT,
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    metadata TEXT DEFAULT '{}',
    version INTEGER DEFAULT 1,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    CHECK (work_directory != ''),
    CHECK (prompt != ''),
    CHECK (length(work_directory) <= 512),
    CHECK (length(prompt) <= 10000),
    CHECK (retry_count >= 0),
    CHECK (max_retries >= 0),
    CHECK (version >= 1)
);

-- 任务历史表
CREATE TABLE IF NOT EXISTS task_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    status TEXT NOT NULL,
    worker_id TEXT,
    changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    details TEXT DEFAULT '{}',
    
    -- 外键约束
    FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE,
    
    -- 约束
    CHECK (status IN ('waiting', 'working', 'completed', 'failed', 'cancelled'))
);

-- 锁表（用于乐观锁）
CREATE TABLE IF NOT EXISTS locks (
    resource_id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    expires_at DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    CHECK (resource_id != ''),
    CHECK (owner_id != '')
);

-- 系统配置表
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    -- 约束
    CHECK (key != ''),
    CHECK (value != '')
);

-- 性能统计表
CREATE TABLE IF NOT EXISTS performance_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    metric_name TEXT NOT NULL,
    metric_value REAL NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    tags TEXT DEFAULT '{}',
    
    -- 约束
    CHECK (metric_name != '')
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_tasks_status_priority ON tasks(status, priority, created_at);
CREATE INDEX IF NOT EXISTS idx_tasks_work_directory ON tasks(work_directory, status);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_worker_id ON tasks(worker_id, status);
CREATE INDEX IF NOT EXISTS idx_tasks_task_id_version ON tasks(task_id, version);
CREATE INDEX IF NOT EXISTS idx_task_history_task_id ON task_history(task_id, changed_at DESC);
CREATE INDEX IF NOT EXISTS idx_locks_expires_at ON locks(expires_at);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_name_timestamp ON performance_metrics(metric_name, timestamp DESC);

-- 创建触发器：自动更新 updated_at
CREATE TRIGGER IF NOT EXISTS update_tasks_updated_at 
    AFTER UPDATE ON tasks
    FOR EACH ROW
BEGIN
    UPDATE tasks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- 创建触发器：清理过期锁
CREATE TRIGGER IF NOT EXISTS cleanup_expired_locks
    AFTER INSERT ON locks
    FOR EACH ROW
BEGIN
    DELETE FROM locks WHERE expires_at < CURRENT_TIMESTAMP;
END;

-- 插入默认系统配置
INSERT OR IGNORE INTO system_config (key, value, description) VALUES 
    ('max_concurrent_tasks', '100', '最大并发任务数'),
    ('default_task_timeout', '3600', '默认任务超时时间（秒）'),
    ('task_cleanup_interval', '3600', '任务清理间隔（秒）'),
    ('enable_metrics', 'true', '是否启用性能指标'),
    ('log_level', 'info', '日志级别'),
    ('api_rate_limit', '1000', 'API请求速率限制（每分钟）'),
    ('max_task_retries', '3', '最大任务重试次数'),
    ('task_result_ttl', '2592000', '任务结果保留时间（秒）');