# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

古诗词学习 App 后端服务 — Axum 0.7 + MySQL (SQLx 0.7) 纯 JSON API 后端，同时服务于管理后台 SPA 和 iOS App。

## Build & Run

```bash
# Build
cargo build

# Run (reads config.toml by default, override with CONFIG_PATH env var)
cargo run

# Run with custom config
CONFIG_PATH=/path/to/config.toml cargo run

# Lint
cargo clippy

# Format
cargo fmt
```

No tests exist yet in the project.

## Architecture

```
src/
├── main.rs         # Entry: init tracing, load config, connect DB, run migrations, seed admin, start server
├── app.rs          # Axum Router assembly — all route definitions and middleware layers
├── config.rs       # TOML config loading (server/database/auth sections)
├── state.rs        # AppState: holds Config + sqlx::MySqlPool (Clone)
├── errors.rs       # AppError enum → axum IntoResponse (JSON errors with Chinese messages)
├── auth.rs         # Argon2 password hash/verify + JWT create/decode + two Axum auth middleware functions
├── models/
│   └── user.rs     # User row, LoginRequest/Response, RegisterRequest, UserInfo, JwtClaims
└── handlers/
    ├── admin/      # /api/v1/admin/* — JSON API for admin SPA (JWT + role=admin)
    │   ├── auth.rs
    │   ├── dashboard.rs
    │   ├── poems.rs
    │   ├── poets.rs
    │   ├── users.rs
    │   ├── import.rs
    │   └── export.rs
    └── api/        # /api/v1/* — JSON API for iOS App (JWT auth)
        ├── auth.rs
        ├── poems.rs
        └── progress.rs
```

## Key Patterns

- **No service layer**: handlers call SQLx directly. Keep it this way unless complexity demands extraction.
- **Auth**: Two middleware functions in `src/auth.rs` — `admin_api_auth_middleware` (checks role=admin) and `api_auth_middleware`. Both extract Bearer JWT, decode, and insert `UserInfo` into request extensions. Handlers access the user via `Extension<UserInfo>`.
- **Error handling**: All handlers return `Result<impl IntoResponse, AppError>`. `AppError` implements `IntoResponse`, mapping each variant to an HTTP status + JSON `{"error": "…"}` body with Chinese messages.
- **Migrations**: SQL files under `migrations/` are embedded via `include_str!` and executed sequentially at startup in `main.rs` — not using SQLx's migration framework.
- **Admin seeding**: On startup, if no user with the configured admin username exists, one is created with the password from `config.toml`.
- **SPA serving**: `frontend/dist/` is served as static files. Any non-API route falls back to `index.html`. In dev, use a separate Vite dev server for the frontend.
- **Password hashing**: Argon2 operations run inside `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- **Pagination**: List endpoints use `page` and `per_page` query params, defaulting to page=1, per_page=20.

# 交流语言

- 使用中文输出

## Backend Coding Guidelines

### Axum 路由参数语法

Axum 0.7 的路径参数使用 `:param` 语法，**不是** `{param}`：

```rust
// WRONG — {param} 是 Axum 0.8+ 语法，0.7 中会被当作字面量路径段，导致路由永远不匹配
.route("/api/v1/admin/poems/{id}", get(handler))

// CORRECT — 0.7 使用 :param
.route("/api/v1/admin/poems/:id", get(handler))
```

**Why**: `{param}` 在 Axum 0.7 中不会报编译错误，但运行时不会匹配任何请求，所有请求都会落到 SPA fallback 返回 HTML。这是一个难以察觉的 bug。

### MySQL JSON 列查询

数据库中 JSON 类型的列不能直接映射到 Rust `String`，需要用 `CAST` 转换：

```rust
// WRONG — content 是 JSON 类型，sqlx 会报类型不匹配
"SELECT content FROM poems WHERE id = ?"

// CORRECT — 显式转为字符串
"SELECT CAST(content AS CHAR) AS content FROM poems WHERE id = ?"
```

**Why**: MySQL 的 JSON 列与 SQLx 的 `String`（映射为 VARCHAR）不兼容，查询时会报 `mismatched types` 错误。列表查询不包含 content 字段所以没问题，但详情查询必须 CAST。

## Frontend Coding Guidelines

### 前端技术栈

- Vue 3 + Vite + TypeScript + Vue Router
- 无 UI 组件库，全部自定义组件
- 样式文件：`src/styles/global.css`（设计令牌、重置、动画）+ `src/styles/components.css`（组件样式）
- 字体：Noto Serif SC（标题 600 + 正文 400），通过 Google Fonts 加载

### 选择字体的原则

中文字体必须优先考虑字符覆盖率（glyph coverage），避免缺字显示为方块：

- **不要用**字符覆盖不全的装饰性字体作为 primary font（如 ZCOOL XiaoWei、Ma Shan Zheng）
- 如果使用装饰性字体，必须在 fallback 链中紧接一个全覆盖字体
- Noto Serif SC 是安全的选择：Google Fonts 加载、完整 CJK 覆盖、衬体风格契合古典主题

```css
/* WRONG — ZCOOL XiaoWei 缺字（如"回"）会显示为方块 */
font-family: 'ZCOOL XiaoWei', serif;

/* CORRECT — 全覆盖字体作为 primary，或确保 fallback 链正确 */
font-family: 'Noto Serif SC', 'STKaiti', 'KaiTi', serif;
```

### sed 批量替换的风险

不要用 `sed -i` 对包含特殊字符的 CSS 文件做批量替换：

```bash
# DANGEROUS — CSS 中单引号、分号、花括号等字符容易导致 sed 意外匹配或清空文件
sed -i '' "s/font-family: 'ZCOOL XiaoWei', serif;/font-family: 'Noto Serif SC', serif; font-weight: 600;/g" components.css

# PREFER — 使用编辑器的 Read + Edit 工具逐文件精确替换
```

**Why**: 上述 sed 命令曾导致 `components.css` 文件被完全清空，丢失了所有按钮、输入框、表格、对话框等组件样式，整个前端样式崩溃。CSS 文件包含大量特殊字符，不适合用 sed 处理。

### 自定义 Toast / Confirm 组件

项目中不使用 Element Plus，通知和确认弹窗通过 DOM 操作实现：

- `src/utils/toast.ts` — 创建 `.toast-container` + 定时移除的 toast 消息
- `src/utils/confirm.ts` — Promise 化的确认对话框，overlay + box 结构

### CSS 设计系统变量

核心设计令牌定义在 `global.css` 的 `:root` 中，组件样式引用变量而非硬编码：

- 颜色：`--ink-deepest` ~ `--ink-overlay`（墨色层次）、`--paper` / `--paper-dim`（文字）、`--vermillion`（朱红）、`--gold`（金色）、`--jade`（翠玉）
- 动画：`--ease-out-expo`、`--ease-spring`、`--duration-fast/base/slow`
- 阴影：`--shadow-card`、`--shadow-dialog`、`--shadow-glow`
