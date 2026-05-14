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
- **SPA serving**: `frontend/dist/` is embedded into the binary at compile time via `rust-embed` + `axum-embed`. In debug mode, files are read from disk (enables frontend hot reload); in release mode, files are embedded in the binary. Any non-API route falls back to `index.html` for Vue Router HTML5 history mode. In dev, use a separate Vite dev server for the frontend.
- **Password hashing**: Argon2 operations run inside `tokio::task::spawn_blocking` to avoid blocking the async runtime.
- **Pagination**: List endpoints use `page` and `per_page` query params, defaulting to page=1, per_page=20.
- **Connection pool**: `MySqlPoolOptions` configured with `max_connections=10`, `acquire_timeout=10s`, `idle_timeout=600s` to prevent pool exhaustion during heavy operations like batch import.

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

### SQLx 参数绑定与 NOT NULL 列

当数据库列为 `NOT NULL` 且有默认值时，SQLx 参数应绑定具体类型（`String`、`u8`）而非 `Option`：

```rust
// WRONG — Option<String> 绑定到 NOT NULL 列，当值为 None 时数据库拒绝
.bind(&category)  // category: Option<String>
.bind(grade)      // grade: Option<u8>

// CORRECT — 解析为具体类型后绑定，空值用默认值替代
let category: String = entry.category
    .as_deref()
    .map(|c| c.trim())
    .filter(|c| !c.is_empty())
    .unwrap_or("")
    .to_string();
let grade: u8 = entry.grade.unwrap_or(0);
.bind(&category)
.bind(grade)
```

**Why**: 大批量导入时，`Option` 类型可能传 `None` 给 `NOT NULL` 列，导致 SQL 执行静默失败（整个 handler 返回空响应）。绑定具体类型可以从根源消除此问题。

### 批量导入性能优化

处理大量数据导入时，避免 N+1 查询：

```rust
// WRONG — 每条记录都查询一次数据库
for entry in entries {
    let poet_id = find_poet(pool, &name, &dynasty).await?; // N 次查询
}

// CORRECT — 预加载到内存 HashMap，仅在缺失时查询/创建
let all_poets: Vec<(u64, String, String)> = sqlx::query_as(
    "SELECT id, name, dynasty FROM poets"
).fetch_all(pool).await?;

let mut poet_cache: HashMap<(String, String), u64> = all_poets
    .into_iter()
    .map(|(id, name, dynasty)| ((name, dynasty), id))
    .collect();

for entry in entries {
    let poet_id = poet_cache.get(&(name, dynasty))
        .copied()
        .unwrap_or_else(|| create_and_cache(...));
}
```

**Why**: 导入数千条诗词时，每条都查询诗人表会导致大量数据库往返。预加载到 HashMap 后，诗人查找降为 O(1)，导入速度显著提升。

### 数据库连接池耗尽防护

长时间运行的批量操作会占用连接，需合理配置连接池：

```rust
// 正确配置 — 限制连接数 + 超时 + 空闲回收
let pool = sqlx::mysql::MySqlPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(std::time::Duration::from_secs(10))
    .idle_timeout(std::time::Duration::from_secs(600))
    .connect(&config.database.url)
    .await?;
```

**Why**: 默认连接池配置（无限制或超长 idle）在批量导入后，大量连接可能长时间不释放，后续普通查询获取不到连接而超时（表现为 500 错误 + 30s 延迟）。`acquire_timeout` 确保快速失败，`idle_timeout` 回收空闲连接。

### 前端长时间请求的超时处理

批量导入等耗时操作需调整前端超时：

```typescript
// 默认 axios 超时 15s 不够，批量导入需要更长
const res = await api.post('/import', form, { timeout: 300000 }) // 5 分钟
```

**Why**: 导入数千条诗词时后端处理时间可能超过 30 秒，默认超时会导致前端报错，但后端实际上已成功完成导入。这类重操作应单独设置更长的超时。

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

## 经验教训

### Askama SSR 不适合复杂交互页面

项目最初使用 Askama 服务端渲染管理后台，但以下问题导致体验不佳：
- 复杂交互（如诗人搜索自动补全）需要大量 JavaScript 注入，与 SSR 模板混写可维护性差
- 每次交互都需要整页刷新或复杂的局部渲染逻辑
- 前后端耦合在同一编译单元中，前端修改需要重新编译 Rust

**结论**：管理后台这类重交互场景，应使用前后端分离（Vue SPA + JSON API），后端只提供 API。

### 大规模数据导入的变量作用域

处理批量导入时，`category`/`grade`/`translation` 等字段的解析必须放在**重复检测之前**：

```rust
// WRONG — 字段在重复检测之后定义，导致重复检测中的回填逻辑无法引用
let existing = check_duplicate(...).await;
// 此时 category/grade 还未定义，无法执行回填 UPDATE

let category = entry.category...; // 太晚了
let grade = entry.grade...;

if let Some(existing_id) = existing {
    // 这里需要 category/grade 但它们还不存在！
}

// CORRECT — 先解析所有字段，再做重复检测
let category = entry.category...;
let grade = entry.grade...;
let translation = ...;

let existing = check_duplicate(...).await;
if let Some(existing_id) = existing {
    // 可以安全使用 category/grade 进行回填
}
```

**Why**: Rust 编译器不会报错（变量作用域看起来没问题），但逻辑上变量的值在使用点还未计算，导致回填逻辑失效。

### Argon2 密码哈希必须 spawn_blocking

Argon2 是 CPU 密集型操作，直接在 async runtime 中执行会阻塞其他任务：

```rust
// WRONG — 阻塞 async runtime
let hash = auth::hash_password(&password)?;

// CORRECT — 在阻塞线程池中执行
let hash = tokio::task::spawn_blocking(move || {
    auth::hash_password(&password)
}).await.map_err(|e| anyhow::anyhow!(e))?.map_err(|e| anyhow::anyhow!(e.to_string()))?;
```

**Why**: Argon2 的计算耗时可达数十毫秒到数百毫秒，直接在 tokio 的 async 任务中执行会导致其他请求排队等待。`spawn_blocking` 将其移到专用线程池，不阻塞 async runtime。

### 前端编译产物嵌入 Rust 二进制

使用 `rust-embed` + `axum-embed` 将 `frontend/dist/` 嵌入到 Rust 二进制中：

```rust
#[derive(Embed, Clone)]
#[folder = "frontend/dist/"]
struct Assets;

let serve_assets = ServeEmbed::<Assets>::with_parameters(
    Some("index.html"),      // fallback_file — SPA 路由回退
    FallbackBehavior::Ok,    // 非 API 路由返回 200 + index.html
    Some("index.html"),      // index_file — 访问 / 时返回此文件
);
```

**特性**：
- Debug 模式从文件系统读取（支持前端 `pnpm dev` 热更新）
- Release 模式编译时嵌入（Docker 镜像只需一个二进制文件）
- 必须开启 `mime-guess` feature，否则 CSS/JS 返回 `application/octet-stream`
- Docker 多阶段构建：先构建前端 → 再编译 Rust（此时 dist 已存在）→ 最终镜像只含二进制 + 配置
