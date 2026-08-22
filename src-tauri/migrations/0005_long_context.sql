-- SP4: 长上下文一致性引擎 — 嵌入向量存储
-- 设计决策：摘要级粒度（章节摘要 + 元素描述），向量存 BLOB，Rust 内存余弦相似度搜索
-- 规模：每项目约 100-250 个向量，无需 sqlite-vec 虚拟表

CREATE TABLE IF NOT EXISTS embeddings (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL,
    source_type  TEXT NOT NULL CHECK (source_type IN ('chapter', 'character', 'storyline', 'worldview')),
    source_id    TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    -- f32 数组序列化为 little-endian bytes；维度由 model 决定（OpenAI: 1536）
    embedding    BLOB NOT NULL,
    dim          INTEGER NOT NULL,
    model        TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    UNIQUE(source_type, source_id)
);

CREATE INDEX IF NOT EXISTS idx_embeddings_project ON embeddings(project_id);
CREATE INDEX IF NOT EXISTS idx_embeddings_source  ON embeddings(source_type, source_id);
