# Linden Novel · SP1 实现计划

- 日期：2026-08-04
- 对应 spec：[2026-08-04-sp1-app-foundation-design.md](../specs/2026-08-04-sp1-app-foundation-design.md)
- 状态：待执行
- 范围：SP1 — 应用骨架与编辑器基座

> 阶段按依赖顺序排列；每阶段含任务（目标 / 涉及文件 / 验证）。阶段尾有「阶段验收」，全过后再进下一阶段。

---

## (一) 阶段总览

| 阶段 | 内容 | 依赖 |
|---|---|---|
| P0 | 项目脚手架与工具链 | — |
| P1 | 数据层（Rust：连接池/迁移/模型/repo） | P0 |
| P2 | 服务层与 IPC（Rust：AppError/service/command） | P1 |
| P3 | 前端 API 层与类型（types/api/stores） | P2 |
| P4 | 三栏布局与项目列表（路由/树/侧栏/主题） | P3 |
| P5 | TipTap 编辑器与自动保存 | P3, P4 |
| P6 | 导入导出 | P2, P5 |
| P7 | 收尾与 DoD 验证 | 全部 |

---

## (二) P0 项目脚手架与工具链

### 1 初始化 Tauri v2 + Vue3 + TS 项目

- 目标：搭建可启动的空壳
- 涉及文件：`package.json`、`vite.config.ts`、`tsconfig.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`src-tauri/src/main.rs`、`src/main.ts`、`src/App.vue`、`index.html`
- 步骤：用 `yarn create tauri-app` 选 Vue + TS；或手动初始化后接入 `@tauri-apps/cli`
- 验证：`yarn tauri dev` 启动空白窗口，无报错

### 2 前端依赖与配置

- 目标：接入 Pinia / Vue Router / Naive UI / UnoCSS
- 涉及文件：`package.json`、`vite.config.ts`（UnoCSS 插件）、`uno.config.ts`、`src/main.ts`（注册 Pinia/Router/NaiveUI）
- 验证：`yarn dev` 前端可跑，NaiveUI 按钮渲染正常

### 3 Rust 依赖

- 目标：加入 sqlx(sqlite, runtime-tokio-rustls)、tokio、serde、serde_json、thiserror、anyhow、tracing、tracing-subscriber、uuid、chrono、reqwest（SP3 用，可暂留）
- 涉及文件：`src-tauri/Cargo.toml`
- 验证：`cargo check` 通过

### 4 工程基础

- 目标：格式化/lint 基线
- 涉及文件：`.editorconfig`、`rustfmt.toml`、`clippy` 配置、`.gitignore`（已有，确认含 `target/`、`node_modules/`、`.superpowers/`）
- 验证：`cargo fmt --check`、`cargo clippy` 无警告

**P0 验收**：`yarn tauri dev` 启动，前后端联调通道通（前端 invoke 一个 hello command 成功）。

---

## (三) P1 数据层（Rust）

### 1 SQLite 连接池与 State

- 目标：初始化 `SqlitePool`，注入 Tauri managed state
- 涉及文件：`src-tauri/src/db/pool.rs`、`src-tauri/src/lib.rs`、`src-tauri/src/config.rs`（DB 文件路径：app data dir 下 `linden.db`）
- 验证：启动时创建库文件，池可获取连接

### 2 迁移

- 目标：建表
- 涉及文件：`src-tauri/src/db/migrations/0001_init.sql`（spec 四.2 全表）、`pool.rs` 启动时 `sqlx::migrate!`
- 验证：启动后表结构存在（`sqlite3` 查 `sqlite_master`）

### 3 领域模型

- 目标：定义结构体 + serde
- 涉及文件：`src-tauri/src/models/{mod,project,volume,chapter,content,settings}.rs`
- 验证：`cargo check`

### 4 repo 层 CRUD

- 目标：每表一个 repo，纯 SQLx 查询
- 涉及文件：`src-tauri/src/db/repo/{project_repo,volume_repo,chapter_repo,content_repo,settings_repo}.rs`
- 覆盖：list/get/create/update/delete + reorder（批量写 `order_index`）+ content get/save
- 验证：Rust 单测（`:memory:` SQLite，跑迁移后测 CRUD + 级联删除 + reorder）

**P1 验收**：repo 单测全绿；级联删除生效；reorder 正确持久化。

---

## (四) P2 服务层与 IPC（Rust）

### 1 AppError

- 涉及文件：`src-tauri/src/error.rs`
- 内容：spec 五.3 枚举 + `Serialize`；DB/IO 错误映射
- 验证：`cargo check`；序列化 JSON 含 `variant`/`message`

### 2 service 层

- 涉及文件：`src-tauri/src/services/{project_service,chapter_service,io_service}.rs`
- 内容：业务编排；`chapter_service::save_content` 计算 `word_count`（中文按字符 + 英文按词）并同事务更新 `chapters.word_count` 与 `chapter_contents`
- 验证：Rust 单测（字数算法用例：纯中文/纯英文/混合/标点）

### 3 command 层与注册

- 涉及文件：`src-tauri/src/commands/{project,volume,chapter,settings}.rs`、`lib.rs` 注册全部 command
- 覆盖：spec 五.2 清单（除导入导出，在 P6）
- 验证：`cargo check`；前端能 invoke 各 command（先用 P3 的 api）

### 4 日志

- 涉及文件：`src-tauri/src/lib.rs`（`tracing_subscriber` 写 app 日志目录文件）
- 验证：操作后日志文件有记录

**P2 验收**：command 全部可调用；保存正文后 `word_count` 正确；错误经 AppError 结构化返回。

---

## (五) P3 前端 API 层与类型

### 1 TS 类型

- 涉及文件：`src/types/{project,chapter,settings}.ts`、`src/types/error.ts`（镜像 `AppError`）
- 验证：`yarn typecheck` 通过

### 2 api 封装

- 涉及文件：`src/api/{project,chapter,settings}.ts`
- 内容：每个 command 一个类型化函数；统一 catch → typed `AppError`
- 验证：在浏览器控制台手动调用返回正确类型

### 3 Pinia stores

- 涉及文件：`src/stores/{project,chapter}.ts`
- 内容：`useProjectStore`（项目/卷列表、当前项目）；`useChapterStore`（章列表、activeChapterId、active 正文、dirty/saving）
- 验证：Vitest 测 list/create/select 流程

**P3 验收**：store 能驱动 UI 数据；类型端到端一致；错误能被捕获。

---

## (六) P4 三栏布局与项目列表

### 1 路由

- 涉及文件：`src/router/index.ts`、`src/views/{ProjectListView,EditorView}.vue`
- 内容：`/` → 项目列表；`/project/:id` → 编辑器
- 验证：路由切换正常

### 2 项目列表页

- 涉及文件：`src/views/ProjectListView.vue`、`src/components/common/ProjectCard.vue`
- 内容：卡片网格，新建/打开/删除（删除二次确认）
- 验证：新建项目入库，列表刷新

### 3 三栏布局

- 涉及文件：`src/components/layout/ThreeColumnLayout.vue`
- 内容：CSS Grid 三栏 + 可拖拽分隔条 + 左右栏折叠按钮
- 验证：拖拽调宽持久化到 `app_settings`；折叠/展开正常

### 4 章节树

- 涉及文件：`src/components/layout/ChapterTree.vue`
- 内容：Naive UI Tree，卷/章层级；拖拽重排（调 reorder command）；右键菜单（新建卷/章·重命名·删除）
- 验证：增删改拖排全部入库；`order_index` 正确

### 5 右侧标签栏与状态栏

- 涉及文件：`src/components/layout/RightSidebar.vue`（n-tabs：本章信息[字数/状态/创建时间] + 故事线/人物/世界观 占位空状态）、`src/components/layout/StatusBar.vue`
- 验证：本章信息随 active chapter 更新

### 6 主题

- 涉及文件：`src/App.vue`（`n-config-provider` + dark theme）、`uno.config.ts`（dark）、主题切换存 `app_settings`
- 验证：明暗切换持久化

**P4 验收**：项目→编辑器→三栏；树全功能；主题切换；空状态下右侧占位清晰。

---

## (七) P5 TipTap 编辑器与自动保存

### 1 编辑器集成

- 涉及文件：`src/components/editor/LindenEditor.vue`、`src/components/layout/EditorPane.vue`
- 内容：`@tiptap/vue-3` + `StarterKit`；加载 `get_chapter_content` → `setContent`；标题栏（章名/状态/字数）
- 验证：打开章即载入正文，可编辑

### 2 自定义节点 SceneBreak

- 涉及文件：`src/components/editor/extensions/sceneBreak.ts`
- 内容：分场分隔线 NodeView（`* * *`）
- 验证：插入/删除分隔线，JSON 中正确序列化

### 3 选区气泡菜单

- 涉及文件：`src/components/editor/BubbleToolbar.vue`
- 内容：粗体/斜体/标题/列表/引用；预留 AI 操作挂载点（SP5）
- 验证：选区弹出，格式生效

### 4 专注/打字机模式

- 涉及文件：`LindenEditor.vue`（CSS + 滚动居中）
- 验证：模式切换，光标垂直居中

### 5 自动保存

- 涉及文件：`src/stores/chapter.ts`（debounce 1.5s）、`EditorPane.vue`（状态指示）
- 内容：`onUpdate` debounce → `save_chapter_content`；切换章前 flush；状态 saving/saved
- 验证：编辑→停顿→已保存；切章不丢内容；保存失败保留 dirty + 提示

### 6 字数统计

- 涉及文件：`src/components/editor/useWordCount.ts`
- 内容：前端实时估值（与 Rust 算法一致）；权威值用保存返回的 `word_count`
- 验证：状态栏字数随输入更新；保存后与树/侧栏一致

**P5 验收**：流畅写作，自动保存可靠，字数一致，模式可用，崩溃最多丢 1.5s。

---

## (八) P6 导入导出

### 1 插件与权限

- 涉及文件：`src-tauri/Cargo.toml`（`tauri-plugin-dialog`、`tauri-plugin-fs`）、`tauri.conf.json`（权限/能力配置）、`lib.rs` 注册插件
- 验证：前端能弹文件对话框

### 2 导出（Rust）

- 涉及文件：`src-tauri/src/services/io_service.rs`、`commands/io.rs`
- 内容：`export_project(project_id, format, path)` → txt/md/json；JSON 含版本号
- 验证：三种格式内容正确；中文无乱码（UTF-8）

### 3 导入（Rust）

- 涉及文件：同上
- 内容：`import_project(path, format)` → json 全量恢复；txt/md 按标题拆章
- 验证：JSON 导入后与原项目数据一致；txt/md 拆章合理

### 4 前端入口

- 涉及文件：`src/views/ProjectListView.vue`（导入按钮）、`src/views/EditorView.vue`（导出菜单）
- 验证：端到端：导出→删除→导入→恢复

**P6 验收**：导出三格式正确；JSON 往返一致；txt/md 导入可用。

---

## (九) P7 收尾与 DoD 验证

### 1 错误处理统一

- 涉及文件：`src/api/`（全局 message）、`LindenEditor.vue`（加载失败回退空文档、保存失败提示）
- 验证：断网/坏数据场景有友好提示

### 2 文档

- 涉及文件：`README.md`（运行/构建/打包说明）、`docs/` 目录索引
- 验证：新成员按 README 可跑起来

### 3 DoD 走查

逐条验证 spec 十一节 DoD：
1. 新建/打开/删除项目，三栏，树增删改拖排 ✓
2. TipTap 写作，1.5s 自动保存，字数，专注/打字机 ✓
3. 导出 TXT/MD/JSON，JSON 导入恢复 ✓
4. 明暗主题 ✓
5. Rust 单测覆盖 repo/service ✓

**P7 验收**：DoD 5 条全过；`yarn tauri build` 产出可执行包。

---

## (十) 执行建议

- 每阶段完成后跑该阶段验收 + `cargo test` + `yarn typecheck`，通过再进下一阶段
- P1/P2（Rust 数据与 IPC）是地基，优先做扎实；P4/P5 是体验核心，可迭代打磨
- 单测与功能同步写，避免 P7 集中补测
- 风险点：TipTap 自定义节点序列化、Tauri v2 权限配置、SQLx 离线模式编译（需 `DATABASE_URL` 或 `sqlx prepare` 离线检查）——遇阻优先查官方文档
