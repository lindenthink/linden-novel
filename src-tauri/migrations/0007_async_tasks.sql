CREATE TABLE IF NOT EXISTS async_tasks (
    id              TEXT PRIMARY KEY,
    task_type       TEXT NOT NULL,   -- 'embed_element', 'embed_chapter', 'sync_embeddings', 'generate_summary'
    project_id      TEXT NOT NULL,
    target_type     TEXT,            -- 'character', 'storyline', 'chapter' 等
    target_id       TEXT,
    content_hash    TEXT,            -- 用于幂等校验（如嵌入内容的 hash）
    payload_json    TEXT,            -- 任务执行所需的额外参数（JSON）
    
    status          TEXT NOT NULL DEFAULT 'pending', -- pending, running, completed, failed, cancelled
    progress_current INTEGER DEFAULT 0,
    progress_total   INTEGER DEFAULT 0,
    
    result_json     TEXT,            -- 成功后的结果（如嵌入数量）
    error_message   TEXT,            -- 失败原因
    
    created_at      TEXT NOT NULL,
    started_at      TEXT,
    completed_at    TEXT,
    
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 索引：用于按项目和状态列表查询
CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON async_tasks(project_id, status);

-- 部分唯一索引：实现幂等提交
-- 确保同一 task_type + target_id + content_hash 组合，在 pending/running 状态下只有一条记录
CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_idempotent 
ON async_tasks (task_type, target_id, content_hash) 
WHERE status IN ('pending', 'running');
