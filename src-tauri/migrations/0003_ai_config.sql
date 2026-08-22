-- SP3: AI 服务层配置表

-- AI Provider 配置表
CREATE TABLE IF NOT EXISTS ai_providers (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    provider_type TEXT NOT NULL,  -- 'openai', 'claude', 'custom'
    base_url     TEXT NOT NULL,
    models_json  TEXT NOT NULL,   -- JSON array of model names
    is_default   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- AI API Key 表（加密存储）
CREATE TABLE IF NOT EXISTS ai_api_keys (
    id            TEXT PRIMARY KEY NOT NULL,
    provider_id   TEXT NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    name          TEXT NOT NULL,
    encrypted_key TEXT NOT NULL,  -- AES-256-GCM 加密后的密钥
    is_default    INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Prompt 模板表
CREATE TABLE IF NOT EXISTS prompt_templates (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    template_type TEXT NOT NULL,  -- 'continue', 'expand', 'polish', 'outline', 'custom'
    content      TEXT NOT NULL,   -- 模板内容，支持 {{variable}} 占位符
    variables_json TEXT,          -- JSON array of variable definitions
    description  TEXT,
    is_builtin   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_ai_api_keys_provider ON ai_api_keys(provider_id);
CREATE INDEX IF NOT EXISTS idx_prompt_templates_type ON prompt_templates(template_type);

-- 插入内置 Prompt 模板
INSERT INTO prompt_templates (id, name, template_type, content, variables_json, description, is_builtin)
VALUES
    ('builtin-continue', '续写', 'continue',
     '请根据以下小说内容，继续写下去：\n\n{{chapter_content}}\n\n请保持文风一致，情节连贯。',
     '["chapter_content"]',
     '根据已有内容续写下一章', 1),

    ('builtin-expand', '扩写', 'expand',
     '请扩写以下段落，增加细节和描写：\n\n{{selected_text}}\n\n要求：\n1. 保持原意不变\n2. 增加环境、心理、动作描写\n3. 字数扩展到原来的 2-3 倍',
     '["selected_text"]',
     '扩写选中的段落', 1),

    ('builtin-polish', '润色', 'polish',
     '请润色以下文字，提升文学性：\n\n{{selected_text}}\n\n要求：\n1. 保持原意\n2. 优化措辞和句式\n3. 增强画面感',
     '["selected_text"]',
     '润色选中的文字', 1),

    ('builtin-outline', '生成大纲', 'outline',
     '请为以下小说设定生成章节大纲：\n\n标题：{{project_title}}\n类型：{{genre}}\n简介：{{summary}}\n\n主要人物：\n{{characters}}\n\n请生成 {{chapter_count}} 章的大纲，每章包含标题和简要内容。',
     '["project_title", "genre", "summary", "characters", "chapter_count"]',
     '根据项目设定生成章节大纲', 1),

    -- 叙事规则：宽松模式（作为「约束程度：宽松」默认规则，用户可在 AI 设置中修改）
    ('builtin-narrative-loose', '叙事规则（宽松）', 'narrative_loose',
     '## 风格与质量控制
- 保持与前文一致的叙事视角和语言风格
- 对话自然，符合角色身份和性格
- 节奏张弛有度，避免平铺直叙
- 如果涉及设定，请贴合世界观，不要自创矛盾设定',
     '[]',
     '宽松约束下的写作风格与质量控制规则', 1),

    -- 叙事规则：严格模式（作为「约束程度：严格」默认规则，用户可在 AI 设置中修改）
    ('builtin-narrative-strict', '叙事规则（严格）', 'narrative_strict',
     '## 叙事规则
- 视角：紧贴主角，不写他不知道的事
- 节奏：短句用于紧张处，长句后接短句，每段≤5句
- 情绪：用动作和生理反应，不用"XX地"或"XX之色"
- 禁用：如同、宛如、仿佛、正是、微微、轻轻、渐渐
- 禁止：开头写环境、人物直接报身份、结尾做总结/点题
- 对话：必须有功能，穿插动作，不同角色说话要有区分度',
     '[]',
     '严格约束下的叙事规则（节奏、视角、措辞、禁用词等）', 1);
