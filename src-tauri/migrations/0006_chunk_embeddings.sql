-- SP5: 切片级嵌入存储
-- embedding_chunks：章节切片元数据 + 向量 BLOB（权威数据源）
-- vec0 虚拟表在 pool.rs 启动后尝试附加（依赖 sqlite-vec 扩展）

CREATE TABLE IF NOT EXISTS embedding_chunks (
    id            TEXT PRIMARY KEY NOT NULL,
    project_id    TEXT NOT NULL,
    chapter_id    TEXT NOT NULL,
    chunk_index   INTEGER NOT NULL,
    chunk_text    TEXT NOT NULL,
    char_count    INTEGER NOT NULL,
    content_hash  TEXT NOT NULL,
    embedding     BLOB NOT NULL,
    dim           INTEGER NOT NULL,
    model         TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(chapter_id, chunk_index)
);

CREATE INDEX IF NOT EXISTS idx_chunks_project ON embedding_chunks(project_id);
CREATE INDEX IF NOT EXISTS idx_chunks_chapter ON embedding_chunks(chapter_id);
