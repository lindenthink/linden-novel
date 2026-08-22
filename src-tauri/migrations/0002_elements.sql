-- SP2: 小说元素管理（Codex）
-- 人物 / 故事线 / 世界观 + 章节-元素关联

CREATE TABLE IF NOT EXISTS characters (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    role         TEXT,
    description  TEXT,
    avatar       TEXT,
    order_index  INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_characters_project ON characters(project_id);

CREATE TABLE IF NOT EXISTS storylines (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    description  TEXT,
    status       TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'abandoned')),
    order_index  INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_storylines_project ON storylines(project_id);

CREATE TABLE IF NOT EXISTS worldview (
    id           TEXT PRIMARY KEY NOT NULL,
    project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    category     TEXT,
    description  TEXT,
    order_index  INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_worldview_project ON worldview(project_id);

CREATE TABLE IF NOT EXISTS chapter_elements (
    id           TEXT PRIMARY KEY NOT NULL,
    chapter_id   TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    element_type TEXT NOT NULL CHECK (element_type IN ('character', 'storyline', 'worldview')),
    element_id   TEXT NOT NULL,
    UNIQUE(chapter_id, element_type, element_id)
);
CREATE INDEX IF NOT EXISTS idx_chapter_elements_chapter ON chapter_elements(chapter_id);
CREATE INDEX IF NOT EXISTS idx_chapter_elements_element ON chapter_elements(element_type, element_id);
