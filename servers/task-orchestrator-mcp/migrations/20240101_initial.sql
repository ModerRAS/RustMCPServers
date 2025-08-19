CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    work_directory TEXT NOT NULL,
    prompt TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 2,
    status INTEGER NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '[]',
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    started_at DATETIME,
    completed_at DATETIME,
    result TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    timeout_seconds INTEGER NOT NULL DEFAULT 3600,
    worker_id TEXT,
    error_message TEXT,
    
    -- Indexes for performance
    INDEX idx_tasks_status (status),
    INDEX idx_tasks_priority (priority),
    INDEX idx_tasks_work_directory (work_directory),
    INDEX idx_tasks_worker_id (worker_id),
    INDEX idx_tasks_created_at (created_at),
    INDEX idx_tasks_status_priority (status, priority, created_at)
);

-- Insert a sample task for testing
INSERT INTO tasks (
    id, work_directory, prompt, priority, status, tags,
    created_at, updated_at, started_at, completed_at,
    result, retry_count, max_retries, timeout_seconds,
    worker_id, error_message
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    '/tmp/test',
    'Test task for validation',
    2,
    0,
    '["test", "sample"]',
    datetime('now'),
    datetime('now'),
    NULL,
    NULL,
    NULL,
    0,
    3,
    3600,
    NULL,
    NULL
);