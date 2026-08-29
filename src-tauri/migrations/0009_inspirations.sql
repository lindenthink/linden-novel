-- 灵感记录（项目内流水式记录，按创建时间倒序，无手动排序）

CREATE TABLE IF NOT EXISTS inspirations (
    id         TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content    TEXT NOT NULL,
    tag        TEXT,
    status     TEXT NOT NULL DEFAULT 'new'
               CHECK (status IN ('new', 'adopted', 'shelved')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_inspirations_project ON inspirations(project_id);
CREATE INDEX IF NOT EXISTS idx_inspirations_status ON inspirations(status);
