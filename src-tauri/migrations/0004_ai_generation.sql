-- AI 生成历史记录表
CREATE TABLE IF NOT EXISTS ai_generation_history (
    id TEXT PRIMARY KEY NOT NULL,
    chapter_id TEXT NOT NULL,
    mode TEXT NOT NULL,
    input_context TEXT NOT NULL,
    output_content TEXT NOT NULL,
    parameters_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_ai_generation_chapter ON ai_generation_history(chapter_id);
CREATE INDEX IF NOT EXISTS idx_ai_generation_mode ON ai_generation_history(mode);
CREATE INDEX IF NOT EXISTS idx_ai_generation_created ON ai_generation_history(created_at DESC);
