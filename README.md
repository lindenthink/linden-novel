# Linden Novel

AI 小说编辑器 — 基于 Rust + Tauri + Vue3 构建的桌面应用。

## 功能特性

- **三栏布局**：章节树 / 编辑器 / 信息面板，可拖拽调整宽度
- **TipTap 富文本编辑器**：支持标题、粗体、斜体、列表、引用等格式
- **自动保存**：编辑后 2 秒自动保存，切换章节前自动 flush
- **项目导入导出**：支持 TXT / Markdown / JSON 三种格式
- **明暗主题**：自动跟随系统或手动切换，设置持久化
- **SQLite 本地存储**：数据完全本地化，无需联网

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust, Tauri v2, SQLx, SQLite, Tokio |
| 前端 | Vue3, TypeScript, Pinia, Vue Router |
| UI | Naive UI, UnoCSS, Carbon Icons |
| 编辑器 | TipTap v2 (ProseMirror) |

## 环境要求

- **Rust**: 1.70+（推荐最新稳定版）
- **Node.js**: 18+
- **Yarn**: 1.22+
- **系统依赖**: [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)

## 快速开始

### 1. 安装依赖

```bash
# 前端依赖
yarn install

# Rust 依赖（首次构建时自动下载）
```

### 2. 开发模式

```bash
yarn tauri dev
```

启动后会自动打开应用窗口，前端支持 HMR 热更新。

### 3. 构建生产版本

```bash
yarn tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：

- Windows: `.msi` / `.nsis` 安装包 + `.exe` 便携版
- macOS: `.dmg` + `.app`
- Linux: `.deb` / `.AppImage`

### 4. 仅构建前端

```bash
yarn build
```

## 项目结构

```
linden-novel/
├── src/                          # 前端源码
│   ├── api/                      # Tauri IPC 调用封装
│   ├── components/
│   │   ├── common/               # 通用组件（ProjectCard）
│   │   ├── editor/               # 编辑器组件（LindenEditor, BubbleToolbar）
│   │   └── layout/               # 布局组件（三栏、章节树、侧栏、状态栏）
│   ├── composables/              # 组合式函数（useTheme）
│   ├── router/                   # 路由配置
│   ├── stores/                   # Pinia 状态管理
│   ├── types/                    # TypeScript 类型定义
│   ├── utils/                    # 工具函数（错误处理）
│   ├── views/                    # 页面视图
│   ├── App.vue
│   └── main.ts
├── src-tauri/                    # 后端源码
│   ├── capabilities/             # Tauri 权限配置
│   ├── migrations/               # 数据库迁移脚本
│   ├── src/
│   │   ├── commands/             # Tauri 命令（IPC 入口）
│   │   ├── db/                   # 数据库连接池
│   │   ├── models/               # 数据模型
│   │   ├── services/             # 业务逻辑层
│   │   ├── error.rs              # 统一错误类型
│   │   ── lib.rs                # 应用入口
│   ├── Cargo.toml
│   └── tauri.conf.json
└── docs/                         # 设计文档
    ├── specs/                    # 功能规格
    └── plans/                    # 实现计划
```

## 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands/` 下创建模块
2. 在 `commands/mod.rs` 中导出
3. 在 `lib.rs` 的 `invoke_handler` 中注册
4. 在 `src/api/` 下创建对应的 TypeScript 封装

### 数据库迁移

在 `src-tauri/migrations/` 下添加新的 SQL 文件，命名格式：`NNNN_description.sql`

### 前端类型同步

Rust models 与 TypeScript types 需保持同步。修改 Rust 模型后，同步更新 `src/types/` 下的对应接口。

## 许可证

MIT
