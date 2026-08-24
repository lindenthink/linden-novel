-- SP6: 伏笔管理（跨章节埋点-回收追踪）
-- 伏笔本质是一对章节关系（埋点章节 + 回收章节），独立于 chapter_elements 多对多关联

CREATE TABLE IF NOT EXISTS foreshadows (
    id                 TEXT PRIMARY KEY NOT NULL,
    project_id         TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title              TEXT NOT NULL,
    description        TEXT,
    importance         TEXT NOT NULL DEFAULT 'normal'
                       CHECK (importance IN ('minor', 'normal', 'major')),
    status             TEXT NOT NULL DEFAULT 'planted'
                       CHECK (status IN ('pending', 'planted', 'resolved', 'abandoned')),
    plant_chapter_id   TEXT REFERENCES chapters(id) ON DELETE SET NULL,
    resolve_chapter_id TEXT REFERENCES chapters(id) ON DELETE SET NULL,
    plant_note         TEXT,
    resolve_note       TEXT,
    order_index        INTEGER NOT NULL DEFAULT 0,
    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_foreshadows_project ON foreshadows(project_id);
CREATE INDEX IF NOT EXISTS idx_foreshadows_plant_chapter ON foreshadows(plant_chapter_id);
CREATE INDEX IF NOT EXISTS idx_foreshadows_resolve_chapter ON foreshadows(resolve_chapter_id);
CREATE INDEX IF NOT EXISTS idx_foreshadows_status ON foreshadows(status);
