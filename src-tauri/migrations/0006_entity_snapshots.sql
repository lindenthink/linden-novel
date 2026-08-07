-- SP4.5: 实体状态快照
-- 追踪角色/故事线在各章节的状态演变

CREATE TABLE IF NOT EXISTS entity_snapshots (
    id              TEXT PRIMARY KEY NOT NULL,
    project_id      TEXT NOT NULL,
    entity_type     TEXT NOT NULL CHECK (entity_type IN ('character', 'storyline')),
    entity_id       TEXT NOT NULL,
    chapter_id      TEXT NOT NULL,
    -- AI 提取的结构化状态（JSON），如: {"status":"alive","location":"北京","role":"主角"}
    state_json      TEXT NOT NULL,
    -- 自然语言摘要
    summary         TEXT NOT NULL,
    -- 与上一快照的变化描述
    changes         TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entity_type, entity_id, chapter_id)
);

CREATE INDEX IF NOT EXISTS idx_entity_snapshots_project ON entity_snapshots(project_id);
CREATE INDEX IF NOT EXISTS idx_entity_snapshots_entity  ON entity_snapshots(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_snapshots_chapter ON entity_snapshots(chapter_id);
