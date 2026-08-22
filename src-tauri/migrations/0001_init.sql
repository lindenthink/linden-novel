-- P1: 初始数据表（projects / volumes / chapters / chapter_contents / app_settings）

CREATE TABLE IF NOT EXISTS projects (
    id           TEXT PRIMARY KEY NOT NULL,
    title        TEXT NOT NULL,
    genre        TEXT,
    summary      TEXT,
    target_words INTEGER,
    settings_json TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS volumes (
    id         TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS chapters (
    id         TEXT PRIMARY KEY NOT NULL,
    volume_id  TEXT NOT NULL REFERENCES volumes(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    status     TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'writing', 'final')),
    word_count INTEGER NOT NULL DEFAULT 0,
    summary    TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_chapters_volume_order ON chapters(volume_id, order_index);
CREATE INDEX IF NOT EXISTS idx_chapters_project      ON chapters(project_id);

CREATE TABLE IF NOT EXISTS chapter_contents (
    chapter_id   TEXT PRIMARY KEY NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
    content_json TEXT NOT NULL DEFAULT '{}',
    content_text TEXT NOT NULL DEFAULT '',
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
