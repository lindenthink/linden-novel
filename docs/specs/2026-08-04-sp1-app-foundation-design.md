# Linden Novel · SP1 应用骨架与编辑器基座 设计

**AI 网文连载编辑器（Rust + Tauri v2 + Vue3）· 第一个子项目**

- 日期：2026-08-04
- 状态：草案（待评审）
- 范围：SP1 — 应用骨架与编辑器基座（不含 AI）
- 作者：[待填写]
- 技术栈：Rust / Tauri v2 / Vue 3.5 + TypeScript / SQLx + SQLite / TipTap

---

## (一) 概述与范围

### 1 项目背景

Linden Novel 是一个面向**中文网文连载**的 AI 桌面编辑器。核心闭环：大纲 → AI 生成章节草稿 → 人工审改定稿 → AI 审稿给修改补丁 → 一键采纳。需支撑百万字级长篇的角色/境界/伏笔/世界观一致性。

### 2 关键决策（已确认）

| 维度 | 决策 |
|---|---|
| 创作场景 | 中文网文连载（爽点/节奏/伏笔/长篇一致性） |
| AI 介入程度 | 半自动生成 + 人工审改（人在环上） |
| AI 接入 | OpenAI 兼容协议统一接入（BYOK），覆盖 DeepSeek/通义/智谱/Kimi/OpenAI/本地 Ollama |
| 数据存储 | 纯本地最简（SQLite + 文件，备份导入导出，无云同步） |
| 整体架构 | **方案 A · Rust 重核**：Rust 独占数据层/AI/RAG/管线，Vue3 纯 UI |

### 3 子项目全景与构建顺序

| # | 子项目 | 依赖 |
|---|---|---|
| **SP1** | 应用骨架与编辑器基座（本 spec） | — |
| SP2 | 小说元素管理（设定集 Codex：人物/故事线/世界观/时间线） | SP1 |
| SP3 | AI 服务层（Provider/流式/密钥/Prompt/上下文组装） | SP1 |
| SP4 | 长上下文一致性引擎（摘要/向量索引 sqlite-vec/实体图/RAG） | SP1,2,3 |
| SP5 | AI 生成（大纲/章节草稿/续写扩写润色/节奏控制） | SP1-4 |
| SP6 | AI 审查（一致性/节奏/爽点/文风/AI痕迹 → 修改补丁） | SP1-4 |

顺序：SP1 →（SP2、SP3 并行）→ SP4 →（SP5、SP6）。

### 4 SP1 目标

交付一个**可用的（无 AI）网文编辑器**地基：项目/卷/章管理、富文本写作、自动保存、三栏布局、导入导出。所有后续子项目依赖此基座。

### 5 非目标（SP1 不做）

- AI 生成/审查/流式（SP3+）
- 角色档案/故事线/世界观实装（SP2；SP1 仅右侧标签占位）
- 向量索引/RAG/嵌入（SP4）
- 云同步、版本历史回滚、多设备协作
- 拼写/语法检查、自动补全

---

## (二) 技术栈

### 1 前端

- Vue 3.5+（`<script setup>` + Composition API）+ TypeScript + Vite 5
- Pinia（状态）/ Vue Router（视图路由）
- TipTap v2（`@tiptap/vue-3`，ProseMirror 内核）—— 选型理由：富格式 + 自定义节点（分场分隔）+ 选区 AI 操作扩展性
- Naive UI（TS 原生、暗色模式、Tree/Tabs/Dialog 开箱即用）+ UnoCSS（原子化排版）
- `@tauri-apps/api`（invoke / event）

### 2 后端（Rust / Tauri v2）

- Tauri v2（插件：dialog / fs / window / os）
- SQLx（编译期校验 SQL、async、SQLite）+ SQLite；迁移 `sqlx migrate`（为 SP4 预留 sqlite-vec 扩展位）
- tokio（异步）/ reqwest（SP3 AI HTTP+SSE）/ serde
- thiserror + anyhow（错误）/ tracing（日志）
- 密钥存储：`tauri-plugin-stronghold` 或系统 keychain（SP3 用，SP1 留接口）

---

## (三) 项目结构

```
linden-novel/
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── lib.rs              # tauri::Builder，注册 command/event
│   │   ├── db/
│   │   │   ├── pool.rs         # SqlitePool 初始化
│   │   │   ├── migrations/     # .sql 迁移文件
│   │   │   └── repo/           # project_repo / chapter_repo ...
│   │   ├── models/             # 领域结构体 (Project, Chapter ...)
│   │   ├── services/           # 业务逻辑 (project_service, chapter_service)
│   │   ├── commands/           # #[tauri::command] 薄包装
│   │   │   ├── project.rs
│   │   │   └── chapter.rs
│   │   ├── error.rs            # AppError → serde
│   │   └── config.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                        # Vue3 前端
│   ├── App.vue  main.ts
│   ├── router/  stores/  views/
│   ├── components/
│   │   ├── layout/             # ThreeColumnLayout, ChapterTree, EditorPane, RightSidebar, StatusBar
│   │   ├── editor/             # TipTap 编辑器 + 扩展
│   │   └── common/
│   ├── api/                    # 类型化 invoke 封装
│   ├── types/                  # 镜像 Rust 模型的 TS 类型
│   └── styles/
├── docs/specs/                 # 设计 spec
├── package.json  vite.config.ts  tsconfig.json
```

---

## (四) 数据模型

### 1 设计要点

- **章元数据与正文分表**：树列表加载轻量，正文按需加载，扛百万字。
- 正文存 `content_json`（TipTap 规范 JSON，结构化、易操作）+ 派生 `content_text`（纯文本，供搜索/字数）。
- `word_count` 由 Rust 从 `content_text` 权威计算（中文按字符 + 英文按词），保存时写库。
- `order_index` 支持树拖拽重排。
- 外键 `ON DELETE CASCADE` 保证删卷删章清理。

### 2 表结构（migration 0001_init.sql）

```sql
CREATE TABLE projects (
    id            TEXT PRIMARY KEY,
    title         TEXT NOT NULL,
    genre         TEXT,
    summary       TEXT,
    target_words  INTEGER,
    settings_json TEXT NOT NULL DEFAULT '{}',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

CREATE TABLE volumes (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE INDEX idx_volumes_project ON volumes(project_id, order_index);

CREATE TABLE chapters (
    id          TEXT PRIMARY KEY,
    volume_id   TEXT NOT NULL REFERENCES volumes(id) ON DELETE CASCADE,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    order_index INTEGER NOT NULL,
    status      TEXT NOT NULL DEFAULT 'draft',  -- draft | writing | final
    word_count  INTEGER NOT NULL DEFAULT 0,
    summary     TEXT,                           -- SP1 不用，SP4 章节摘要
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
CREATE INDEX idx_chapters_volume ON chapters(volume_id, order_index);

CREATE TABLE chapter_contents (
    chapter_id   TEXT PRIMARY KEY REFERENCES chapters(id) ON DELETE CASCADE,
    content_json TEXT NOT NULL,                 -- TipTap JSON
    content_text TEXT NOT NULL DEFAULT '',      -- 派生纯文本
    updated_at   TEXT NOT NULL
);

CREATE TABLE app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### 3 字段约定

- 主键：TEXT（UUID v4 字符串）
- 时间：TEXT（RFC3339 UTC）
- `status`：枚举字符串 `draft` / `writing` / `final`
- `app_settings`：KV，存主题偏好、窗口状态等；SP3 的 Provider 配置亦存此（或单独表，SP3 定）

---

## (五) IPC 与服务层架构

### 1 分层

```
commands/  →  services/  →  db/repo/   (SQLx)
 (校验输入)   (业务逻辑)    (SQL 查询)
```

- `SqlitePool` 作为 Tauri managed state 注入 command。
- command 返回 `Result<T, AppError>`；`AppError` 实现 `Serialize`，前端拿结构化错误。

### 2 command 清单（SP1）

| 领域 | command | 说明 |
|---|---|---|
| 项目 | `list_projects` `get_project` `create_project` `update_project` `delete_project` | |
| 卷 | `list_volumes` `create_volume` `update_volume` `delete_volume` `reorder_volumes` | |
| 章 | `list_chapters(volume_id)` `get_chapter` `create_chapter` `update_chapter_meta` `delete_chapter` `reorder_chapters` | |
| 正文 | `get_chapter_content(chapter_id)` `save_chapter_content(chapter_id, content_json, content_text)` | Rust 计算 word_count |
| 设置 | `get_setting(key)` `set_setting(key, value)` | |
| 导入导出 | `export_project(project_id, format, path)` `import_project(path, format)` | format: txt\|md\|json |

> **事件**：SP1 极简，保存由前端 debounce 主动 invoke、返回值即结果，无需 event。事件机制留到 SP3（`ai:chunk` 流式）。

### 3 AppError

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
pub enum AppError {
    #[error("数据库错误: {0}")]     Db(String),
    #[error("未找到: {0}")]         NotFound(String),
    #[error("参数错误: {0}")]       Validation(String),
    #[error("IO 错误: {0}")]        Io(String),
    #[error("内部错误: {0}")]       Internal(String),
}
```

command 层 `?` 传播；service 层把底层错误转成对应变体；DB 约束冲突 → Validation 友好提示。

---

## (六) 编辑器集成

### 1 TipTap 配置

- `@tiptap/vue-3` + `StarterKit`（标题/粗斜体/列表/引用/代码块）
- 自定义节点：
  - `SceneBreak`——章节内分场分隔线（网文常用 `* * *`），自定义 NodeView
  - `ElementMention`（@提及角色/设定）——SP1 只留扩展位，SP2 实装
- 写作体验：专注模式 / 打字机模式（光标垂直居中）/ 章节标题输入区
- 选区气泡菜单（BubbleMenu）：基础格式按钮；为 SP5 选区 AI 操作（续写/润色/扩写）预留挂载点

### 2 内容进出

- 加载：`get_chapter_content` → `content_json` → `editor.commands.setContent(json)`
- 保存：`editor.getJSON()` → `content_json`；`editor.getText()` → `content_text`

---

## (七) 自动保存

- 监听 `onUpdate`，**debounce 1.5s** 无新改动 → 调 `save_chapter_content`
- 状态指示器（Pinia）：`saving` / `saved`，标题栏显示
- **切换章节前**：若当前章 dirty，先 flush 保存再加载新章
- 单机单用户，无并发冲突
- 崩溃恢复：SP1 轻量版——debounce 落库已足够（最多丢 1.5s）；完整草稿恢复留作增强

---

## (八) 三栏布局 UI（布局 A）

```
┌─────────────┬────────────────────────┬──────────────┐
│  章节树      │  编辑器                  │  右侧标签栏   │
│ ChapterTree │  EditorPane             │ RightSidebar │
├─────────────┼────────────────────────┼──────────────┤
│ 项目▾       │  第十二章 …              │ 故事线/人物/   │
│ ├第一卷     │  ─────────────          │ 世界观(SP2)   │
│ │ ├第1章    │  正文区…                 │ ─────────    │
│ │ └第2章    │                         │ 本章信息      │
│ └第二卷     │  状态栏: 3120字·已保存    │ 字数/状态     │
└─────────────┴────────────────────────┴──────────────┘
```

- **三栏**：CSS Grid + 可拖拽分隔条调宽，左右栏可折叠切沉浸写作
- **ChapterTree**（Naive UI Tree）：拖拽重排（写 `order_index`）、右键菜单（新建卷/章·重命名·删除）
- **EditorPane**：标题栏（章名+保存状态+字数）+ TipTap + 选区气泡
- **RightSidebar**（n-tabs）：SP1 放「本章信息」面板（字数/状态/创建时间；摘要字段 SP4 启用）；故事线/人物/世界观 空状态占位，SP2 实装
- **StatusBar**（底栏）：字数、保存状态、光标位置
- **主题**：明/暗，`n-config-provider` + UnoCSS dark

### 1 路由与视图

- `/` → 项目列表页（新建/打开/删除项目卡片）
- `/project/:id` → 编辑器视图（三栏）

### 2 Pinia store

- `useProjectStore`：当前项目、卷列表
- `useChapterStore`：章列表、activeChapterId、active 正文、dirty/saving
- 编辑器实例 ref + 实时字数（估值）

---

## (九) 导入导出与备份

- **导出**：全书 → TXT / Markdown / JSON 全量备份
  - JSON：结构化全量（projects+volumes+chapters+contents + 版本号），用于备份恢复
  - Markdown：`# 卷` / `## 章` / 正文，按层级拆
  - TXT：纯文本，章间分隔
- **导入**：JSON 备份恢复整项目；TXT/MD 按标题分隔符拆为新章
- 实现：`tauri-plugin-dialog`（文件对话框）+ `tauri-plugin-fs`（读写）

---

## (十) 错误处理与测试

### 1 错误处理

- **Rust**：`AppError`（见 五.3）；`tracing` 日志写 app 日志目录
- **前端**：`api/` 统一 catch → typed AppError → Naive UI message 提示；编辑器加载失败回退空文档；保存失败保留 dirty + 提示重试

### 2 测试

- **Rust 单测**：repo 层（`:memory:` SQLite）、service 层（字数计算、导入解析、导出序列化）
- **前端**：Vitest 测 store 逻辑/字数算法；Vue Test Utils 测关键组件
- **E2E**：SP1 先手动验证核心流程，自动化（tauri-driver）后续补

---

## (十一) 交付里程碑（DoD）

1. 新建/打开/删除项目，三栏布局，章节树增删改拖排
2. TipTap 写作，1.5s 自动保存，字数统计，专注/打字机模式
3. 导出 TXT/MD/JSON，JSON 导入恢复
4. 明暗主题
5. Rust 单测覆盖 repo/service 关键逻辑

---

## (十二) 后续子项目衔接

- **SP2**：在 `RightSidebar` 实装故事线/人物/世界观面板；新增元素表；`ElementMention` 节点；元素与章节双向链接
- **SP3**：`app_settings`/独立表存 Provider 配置；reqwest + SSE 流式经 Tauri event；密钥 keychain
- **SP4**：`chapters.summary` 填充；新增嵌入向量表（sqlite-vec）；实体状态快照；RAG 检索
- **SP5/SP6**：编辑器选区气泡挂 AI 操作；审稿补丁应用回 TipTap JSON
