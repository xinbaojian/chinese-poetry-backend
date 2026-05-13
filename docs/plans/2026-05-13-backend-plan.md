# Chinese Poetry Backend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust backend management platform for the Chinese Poetry iOS app, with admin web UI (Askama SSR) and App REST API (JSON).

**Architecture:** Single Axum 0.7 server serving two interfaces — admin pages rendered via Askama templates with session auth, and REST JSON API endpoints with JWT auth for the iOS app. MySQL for persistence, SQLx for async database access.

**Tech Stack:** Rust 2021, Axum 0.7, SQLx 0.7 + MySQL, Askama 0.12, Argon2, JWT, Tower middleware

**Reference:** Existing calendar-manager project at `/Users/xinbaojian/workspace/rust/calendar-manager` uses the same patterns. Follow its conventions for error handling, state management, repository pattern, and template structure.

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `askama.toml`
- Create: `config.toml`
- Create: `.gitignore`
- Create: `src/main.rs` (minimal hello world)
- Create: `src/app.rs`
- Create: `src/state.rs`
- Create: `src/config.rs`

**Step 1: Initialize project**

```bash
cd /Users/xinbaojian/workspace/rust/chinese-poetry-backend
git init
```

**Step 2: Create Cargo.toml**

```toml
[package]
name = "chinese-poetry-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "trace"] }
tower-cookies = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio", "mysql", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
askama = "0.12"
askama_axum = "0.4"
anyhow = "1"
thiserror = "1"
argon2 = "0.5"
jsonwebtoken = "9"
password-hash = { version = "0.5", features = ["getrandom"] }
uuid = { version = "1", features = ["v4"] }
```

**Step 3: Create askama.toml**

```toml
[general]
dirs = ["templates"]
```

**Step 4: Create config.toml**

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "mysql://root:@localhost:3306/chinese_poetry"

[auth]
jwt_secret = "change-this-in-production"
jwt_exp_hours = 72
admin_username = "admin"
admin_password = "admin123"
session_secret = "change-this-session-secret"
```

**Step 5: Create .gitignore**

```
/target
config.toml
```

**Step 6: Create src/config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_exp_hours")]
    pub jwt_exp_hours: u64,
    pub admin_username: String,
    pub admin_password: String,
    pub session_secret: String,
}

fn default_host() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 3000 }
fn default_jwt_secret() -> String { "change-this-in-production".to_string() }
fn default_jwt_exp_hours() -> u64 { 72 }

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    let config_path = std::env::var("CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| path.to_path_buf());
    let content = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

**Step 7: Create src/state.rs**

```rust
use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: sqlx::MySqlPool,
}
```

**Step 8: Create src/app.rs**

```rust
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Chinese Poetry Backend" }))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
```

**Step 9: Create src/main.rs**

```rust
mod app;
mod config;
mod state;

use std::path::Path;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chinese_poetry_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::load_config(Path::new("config.toml"))?;
    let pool = sqlx::MySqlPool::connect(&config.database.url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let state = state::AppState {
        config: config.clone(),
        db: pool,
    };

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);

    let app = app::create_app(state);
    axum::serve(listener, app).await?;

    Ok(())
}
```

**Step 10: Verify it compiles**

```bash
cargo build
```
Expected: Compiles with warnings about unused imports, no errors.

**Step 11: Commit**

```bash
git add -A
git commit -m "feat: project scaffolding with Axum + SQLx + Askama"
```

---

### Task 2: Error Handling

**Files:**
- Create: `src/errors.rs`

**Step 1: Create src/errors.rs**

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use askama::Template;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "数据库操作失败".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, "认证已过期，请重新登录".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "请先登录".to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Duplicate(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!(error = %msg, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误".to_string())
            }
            AppError::Io(e) => {
                tracing::error!(error = %e, "IO error");
                (StatusCode::INTERNAL_SERVER_ERROR, "IO 错误".to_string())
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

Note: `AppError` implements `IntoResponse` for both JSON API responses and can be used in Askama handlers. For admin pages that need error HTML, handlers should catch errors and render templates directly.

**Step 2: Add `mod errors;` to src/main.rs**

**Step 3: Verify compilation**

```bash
cargo build
```

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: add error handling with AppError enum"
```

---

### Task 3: Database Migrations

**Files:**
- Create: `migrations/20260513000001_create_users.sql`
- Create: `migrations/20260513000002_create_poets.sql`
- Create: `migrations/20260513000003_create_poems.sql`
- Create: `migrations/20260513000004_create_learning_records.sql`
- Create: `migrations/20260513000005_create_review_history.sql`
- Create: `migrations/20260513000006_create_settings.sql`

**Step 1: Create all migration files**

`migrations/20260513000001_create_users.sql`:
```sql
CREATE TABLE IF NOT EXISTS users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(64) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role ENUM('admin', 'user') NOT NULL DEFAULT 'user',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

`migrations/20260513000002_create_poets.sql`:
```sql
CREATE TABLE IF NOT EXISTS poets (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    dynasty VARCHAR(32) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_name_dynasty (name, dynasty)
);
```

`migrations/20260513000003_create_poems.sql`:
```sql
CREATE TABLE IF NOT EXISTS poems (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    poet_id BIGINT UNSIGNED NOT NULL,
    dynasty VARCHAR(32) NOT NULL DEFAULT '',
    category VARCHAR(64) NOT NULL DEFAULT '',
    grade TINYINT UNSIGNED NOT NULL DEFAULT 0,
    content JSON NOT NULL,
    translation TEXT,
    source_id VARCHAR(128),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_category (category),
    INDEX idx_grade (grade),
    INDEX idx_dynasty (dynasty),
    INDEX idx_source_id (source_id),
    FOREIGN KEY (poet_id) REFERENCES poets(id)
);
```

`migrations/20260513000004_create_learning_records.sql`:
```sql
CREATE TABLE IF NOT EXISTS learning_records (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id BIGINT UNSIGNED NOT NULL,
    poem_id BIGINT UNSIGNED NOT NULL,
    mastery_level ENUM('proficient', 'fair', 'weak') NOT NULL DEFAULT 'weak',
    review_count INT UNSIGNED NOT NULL DEFAULT 0,
    next_review_date DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_user_poem (user_id, poem_id),
    INDEX idx_user_next_review (user_id, next_review_date),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (poem_id) REFERENCES poems(id)
);
```

`migrations/20260513000005_create_review_history.sql`:
```sql
CREATE TABLE IF NOT EXISTS review_history (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    learning_record_id BIGINT UNSIGNED NOT NULL,
    reviewed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    mastery_level ENUM('proficient', 'fair', 'weak') NOT NULL,
    FOREIGN KEY (learning_record_id) REFERENCES learning_records(id) ON DELETE CASCADE
);
```

`migrations/20260513000006_create_settings.sql`:
```sql
CREATE TABLE IF NOT EXISTS settings (
    `key` VARCHAR(128) NOT NULL PRIMARY KEY,
    `value` VARCHAR(512) NOT NULL
);
```

**Step 2: Create the MySQL database**

```bash
mysql -u root -e "CREATE DATABASE IF NOT EXISTS chinese_poetry CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
```

**Step 3: Run migrations and verify**

```bash
cargo build
# Migrations run automatically on startup via sqlx::migrate!
```

**Step 4: Commit**

```bash
git add migrations/
git commit -m "feat: add database migration files"
```

---

### Task 4: User Model and Auth Utilities

**Files:**
- Create: `src/models/mod.rs`
- Create: `src/models/user.rs`
- Create: `src/auth.rs`

**Step 1: Create src/models/user.rs**

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserInfo {
    pub id: u64,
    pub username: String,
    pub role: String,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}
```

**Step 2: Create src/models/mod.rs**

```rust
pub mod user;
```

**Step 3: Create src/auth.rs**

Password hashing (Argon2), JWT create/verify, and auth middleware for both admin sessions and API JWT.

```rust
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::rand_core::OsRng;

use crate::errors::{AppError, AppResult};
use crate::models::user::{JwtClaims, UserInfo};
use crate::state::AppState;

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn create_jwt(user: &UserInfo, secret: &str, exp_hours: u64) -> AppResult<String> {
    let now = Utc::now();
    let claims = JwtClaims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        iat: now.timestamp() as usize,
        exp: (now.timestamp() + (exp_hours as i64 * 3600)) as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn decode_jwt(token: &str, secret: &str) -> AppResult<JwtClaims> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::InvalidToken)?;
    Ok(token_data.claims)
}

pub async fn api_auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::InvalidToken)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InvalidToken);
    }

    let token = &auth_header[7..];
    let claims = decode_jwt(token, &state.config.auth.jwt_secret)?;

    let user_id: u64 = claims.sub.parse().map_err(|_| AppError::InvalidToken)?;
    let user_info = UserInfo {
        id: user_id,
        username: claims.username.clone(),
        role: claims.role.clone(),
    };

    request.extensions_mut().insert(user_info);
    Ok(next.run(request).await)
}
```

**Step 4: Add modules to main.rs**

```rust
mod models;
mod auth;
```

**Step 5: Verify compilation**

```bash
cargo build
```

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add user model, JWT auth utilities, and Argon2 password hashing"
```

---

### Task 5: Poet and Poem Models

**Files:**
- Create: `src/models/poet.rs`
- Create: `src/models/poem.rs`
- Create: `src/models/learning_record.rs`
- Modify: `src/models/mod.rs`

**Step 1: Create src/models/poet.rs**

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Poet {
    pub id: u64,
    pub name: String,
    pub dynasty: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePoet {
    pub name: String,
    pub dynasty: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePoet {
    pub name: Option<String>,
    pub dynasty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryPoets {
    pub keyword: Option<String>,
    pub dynasty: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
```

**Step 2: Create src/models/poem.rs**

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Poem {
    pub id: u64,
    pub title: String,
    pub poet_id: u64,
    pub dynasty: String,
    pub category: String,
    pub grade: u8,
    pub content: String, // JSON string of paragraphs array
    pub translation: Option<String>,
    pub source_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePoem {
    pub title: String,
    pub poet_id: u64,
    pub dynasty: String,
    pub category: Option<String>,
    pub grade: Option<u8>,
    pub paragraphs: Vec<String>,
    pub translation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePoem {
    pub title: Option<String>,
    pub poet_id: Option<u64>,
    pub dynasty: Option<String>,
    pub category: Option<String>,
    pub grade: Option<u8>,
    pub paragraphs: Option<Vec<String>>,
    pub translation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryPoems {
    pub keyword: Option<String>,
    pub dynasty: Option<String>,
    pub category: Option<String>,
    pub grade: Option<u8>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PoemWithPoet {
    pub id: u64,
    pub title: String,
    pub poet_id: u64,
    pub poet_name: String,
    pub dynasty: String,
    pub category: String,
    pub grade: u8,
    pub content: String,
    pub translation: Option<String>,
    pub created_at: NaiveDateTime,
}
```

**Step 3: Create src/models/learning_record.rs**

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LearningRecord {
    pub id: u64,
    pub user_id: u64,
    pub poem_id: u64,
    pub mastery_level: String,
    pub review_count: u32,
    pub next_review_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ReviewHistoryEntry {
    pub id: u64,
    pub learning_record_id: u64,
    pub reviewed_at: NaiveDateTime,
    pub mastery_level: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncLearningRequest {
    pub records: Vec<SyncRecord>,
}

#[derive(Debug, Deserialize)]
pub struct SyncRecord {
    pub poem_id: u64,
    pub mastery_level: String,
    pub review_count: u32,
    pub next_review_date: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct LearningRecordWithPoem {
    #[serde(flatten)]
    pub record: LearningRecord,
    pub poem_title: String,
    pub poet_name: String,
}
```

**Step 4: Update src/models/mod.rs**

```rust
pub mod user;
pub mod poet;
pub mod poem;
pub mod learning_record;
```

**Step 5: Verify compilation**

```bash
cargo build
```

**Step 6: Commit**

```bash
git add -A
git commit -m "feat: add poet, poem, and learning record models"
```

---

### Task 6: Templates - Base Layout and Login

**Files:**
- Create: `templates/base.html`
- Create: `templates/login.html`
- Create: `templates/admin/dashboard.html`
- Create: `templates/admin/poems.html`
- Create: `templates/admin/poem_form.html`
- Create: `templates/admin/poets.html`
- Create: `templates/admin/poet_form.html`
- Create: `templates/admin/users.html`
- Create: `templates/admin/import.html`
- Create: `templates/admin/export.html`
- Create: `templates/partials/pagination.html`
- Create: `templates/partials/flash.html`
- Create: `static/css/style.css`

**Step 1: Create templates/base.html**

A clean admin layout with sidebar navigation. Use inline CSS (no external dependencies) for simplicity. Include a sidebar with nav links, main content area, and flash message support.

Key elements:
- Sidebar with: 首页, 诗词管理, 诗人管理, 用户管理, 批量导入, 数据导出, 退出
- Main content block
- Flash message block (for success/error notifications)
- HTML head with meta charset utf-8, viewport

**Step 2: Create templates/login.html**

Extends base.html but without sidebar. Centered login form with username and password fields.

**Step 3: Create templates/admin/dashboard.html**

Simple dashboard showing counts: total poems, total poets, total users, total learning records.

Template struct:
```rust
#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct DashboardTemplate {
    pub total_poems: i64,
    pub total_poets: i64,
    pub total_users: i64,
    pub total_records: i64,
}
```

**Step 4: Create all other admin template files with placeholder content**

Each template extends base.html and will be filled in as handlers are built.

**Step 5: Create static/css/style.css**

Minimal admin CSS: sidebar layout, form styling, table styling, button styling, pagination, flash messages. Use a clean blue/white color scheme.

**Step 6: Verify Askama templates compile**

Add template structs to src and run `cargo build`.

**Step 7: Commit**

```bash
git add templates/ static/
git commit -m "feat: add Askama templates for admin UI"
```

---

### Task 7: Admin Auth - Login and Session

**Files:**
- Create: `src/handlers/mod.rs`
- Create: `src/handlers/admin/mod.rs`
- Create: `src/handlers/admin/auth.rs`
- Create: `src/handlers/admin/dashboard.rs`
- Modify: `src/app.rs` (add admin routes)

**Step 1: Create src/handlers/admin/auth.rs**

Admin login page handler (GET /admin/login renders form, POST /admin/login validates credentials and sets session cookie).

- GET: Render login.html template
- POST: Look up user by username, verify password with Argon2, check role is 'admin', set cookie with user_id
- Cookie-based session: store user_id in a signed cookie using tower-cookies

**Step 2: Create src/handlers/admin/dashboard.rs**

GET /admin/ — fetch counts from DB, render dashboard template.

**Step 3: Create admin auth middleware**

Check for valid session cookie, redirect to /admin/login if missing or invalid.

**Step 4: Wire up routes in src/app.rs**

```rust
// Public admin routes (no auth)
let admin_public = Router::new()
    .route("/admin/login", get(admin_login_page).post(admin_login));

// Protected admin routes (session auth)
let admin_protected = Router::new()
    .route("/admin/", get(dashboard))
    .route("/admin/logout", post(admin_logout))
    // ... more routes
    .layer(middleware::from_fn_with_state(state.clone(), admin_auth_middleware));
```

**Step 5: Add seed admin user creation in main.rs**

On startup, check if admin user exists. If not, create one with config credentials.

**Step 6: Test manually**

```bash
cargo run
# Open http://127.0.0.1:3000/admin/login
# Login with admin / admin123
# Should see dashboard
```

**Step 7: Commit**

```bash
git add -A
git commit -m "feat: admin login with session cookie authentication"
```

---

### Task 8: Admin Poet CRUD

**Files:**
- Create: `src/handlers/admin/poets.rs`
- Modify: `src/app.rs` (add poet routes)
- Modify: `templates/admin/poets.html` (list with pagination)
- Modify: `templates/admin/poet_form.html` (create/edit form)

**Step 1: Implement poet list handler**

GET /admin/poets — query poets with optional search (keyword, dynasty), paginate (20 per page), render poets.html.

**Step 2: Implement poet create handler**

GET /admin/poets/new — render empty form.
POST /admin/poets/new — validate input, insert into DB, redirect to list with success flash.

**Step 3: Implement poet edit handler**

GET /admin/poets/{id}/edit — render form with existing data.
POST /admin/poets/{id}/edit — validate, update DB, redirect.

**Step 4: Implement poet delete handler**

POST /admin/poets/{id}/delete — delete from DB, redirect to list.

**Step 5: Wire routes**

```rust
.route("/admin/poets", get(poets_list))
.route("/admin/poets/new", get(poet_new_page).post(poet_create))
.route("/admin/poets/{id}/edit", get(poet_edit_page).post(poet_update))
.route("/admin/poets/{id}/delete", post(poet_delete))
```

**Step 6: Test manually**

Create, edit, delete poets through the admin UI.

**Step 7: Commit**

```bash
git add -A
git commit -m "feat: admin poet CRUD with search and pagination"
```

---

### Task 9: Admin Poem CRUD

**Files:**
- Create: `src/handlers/admin/poems.rs`
- Modify: `src/app.rs` (add poem routes)
- Modify: `templates/admin/poems.html` (list with filters)
- Modify: `templates/admin/poem_form.html` (create/edit form)

**Step 1: Implement poem list handler**

GET /admin/poems — join with poets table for poet_name, filter by dynasty/category/grade/keyword, paginate.

**Step 2: Implement poem create handler**

GET /admin/poems/new — render form with poet dropdown.
POST /admin/poems/new — validate, serialize paragraphs to JSON, insert.

**Step 3: Implement poem edit handler**

GET /admin/poems/{id}/edit — render form with existing data.
POST /admin/poems/{id}/edit — validate, update.

**Step 4: Implement poem delete handler**

POST /admin/poems/{id}/delete — delete, redirect.

**Step 5: Wire routes**

```rust
.route("/admin/poems", get(poems_list))
.route("/admin/poems/new", get(poem_new_page).post(poem_create))
.route("/admin/poems/{id}/edit", get(poem_edit_page).post(poem_update))
.route("/admin/poems/{id}/delete", post(poem_delete))
```

**Step 6: Test manually**

CRUD poems through admin UI, verify content JSON is stored correctly.

**Step 7: Commit**

```bash
git add -A
git commit -m "feat: admin poem CRUD with filters and pagination"
```

---

### Task 10: Batch Import

**Files:**
- Create: `src/handlers/admin/import.rs`
- Create: `src/services/import_service.rs`
- Create: `src/services/mod.rs`
- Modify: `src/app.rs` (add import route)
- Modify: `templates/admin/import.html` (upload form + progress)

**Step 1: Create import service**

Parse uploaded JSON file matching the existing data format:
```json
[{"title": "...", "author": "...", "paragraphes": ["..."], "id": "..."}]
```

Logic:
1. Parse JSON array
2. For each entry: find or create poet by name, then create poem (skip if source_id already exists)
3. Return counts: imported, skipped, failed

**Step 2: Create import handler**

GET /admin/import — render upload form.
POST /admin/import — accept multipart file upload, call import service, render results.

**Step 3: Handle the JSON format variations**

The existing data source may have different field names (`paragraphes` vs `paragraphs`). Handle both.

**Step 4: Test with sample data**

Create a small test JSON file and import it through the admin UI.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: batch import poems from JSON files"
```

---

### Task 11: API Auth (Register/Login)

**Files:**
- Create: `src/handlers/api/mod.rs`
- Create: `src/handlers/api/auth.rs`
- Modify: `src/app.rs` (add API routes)

**Step 1: Implement POST /api/v1/auth/register**

Validate username (3-64 chars) and password (6+ chars). Check uniqueness. Hash password with Argon2. Insert into users table with role 'user'. Return JWT token.

**Step 2: Implement POST /api/v1/auth/login**

Look up user by username, verify password, return JWT token and user info.

**Step 3: Wire routes**

```rust
let api_public = Router::new()
    .route("/api/v1/auth/register", post(api_register))
    .route("/api/v1/auth/login", post(api_login));
```

**Step 4: Test with curl**

```bash
curl -X POST http://127.0.0.1:3000/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123"}'

curl -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123"}'
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: API user registration and login with JWT"
```

---

### Task 12: API Poems Endpoints

**Files:**
- Create: `src/handlers/api/poems.rs`
- Modify: `src/app.rs` (add poem API routes)

**Step 1: Implement GET /api/v1/poems**

Paginated poem list with filters (dynasty, category, grade, keyword). Return JSON array with poem data including poet name.

**Step 2: Implement GET /api/v1/poems/{id}**

Single poem detail with content parsed from JSON string.

**Step 3: Wire routes (protected with JWT auth)**

```rust
let api_protected = Router::new()
    .route("/api/v1/poems", get(api_list_poems))
    .route("/api/v1/poems/{id}", get(api_get_poem))
    .route("/api/v1/progress", get(api_get_progress).post(api_sync_progress))
    .route("/api/v1/progress/due", get(api_get_due_reviews))
    .layer(middleware::from_fn_with_state(state.clone(), api_auth_middleware));
```

**Step 4: Test with curl**

```bash
TOKEN="your-jwt-token"
curl http://127.0.0.1:3000/api/v1/poems -H "Authorization: Bearer $TOKEN"
curl http://127.0.0.1:3000/api/v1/poems/1 -H "Authorization: Bearer $TOKEN"
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: API poem list and detail endpoints"
```

---

### Task 13: API Learning Progress Sync

**Files:**
- Create: `src/handlers/api/progress.rs`
- Modify: `src/app.rs` (add progress routes)

**Step 1: Implement GET /api/v1/progress**

Return all learning records for the authenticated user, joined with poem title and poet name.

**Step 2: Implement POST /api/v1/progress**

Accept batch sync request with array of records. For each record:
- If user+poem combo exists: compare updated_at, keep newer version
- If not exists: insert new record
- Insert review history entries

**Step 3: Implement GET /api/v1/progress/due**

Return learning records where next_review_date <= now, for the authenticated user.

**Step 4: Test with curl**

```bash
# Sync progress
curl -X POST http://127.0.0.1:3000/api/v1/progress \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"records":[{"poem_id":1,"mastery_level":"fair","review_count":3,"next_review_date":"2026-05-15T10:00:00","updated_at":"2026-05-13T10:00:00"}]}'

# Get all progress
curl http://127.0.0.1:3000/api/v1/progress -H "Authorization: Bearer $TOKEN"

# Get due reviews
curl http://127.0.0.1:3000/api/v1/progress/due -H "Authorization: Bearer $TOKEN"
```

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: API learning progress sync with conflict resolution"
```

---

### Task 14: Admin User Management

**Files:**
- Create: `src/handlers/admin/users.rs`
- Modify: `src/app.rs` (add user routes)
- Modify: `templates/admin/users.html`

**Step 1: Implement user list**

GET /admin/users — list all users with pagination, show username, role, created_at, learning record count.

**Step 2: Implement user delete**

POST /admin/users/{id}/delete — prevent deleting self, delete user and cascade learning records.

**Step 3: Test manually**

View user list, delete a test user.

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: admin user management with list and delete"
```

---

### Task 15: Data Export

**Files:**
- Create: `src/handlers/admin/export.rs`
- Create: `src/services/export_service.rs`
- Modify: `src/app.rs` (add export route)
- Modify: `templates/admin/export.html`

**Step 1: Create export page**

GET /admin/export — render form with options: format (CSV/JSON), user filter (all or specific user), date range.

**Step 2: Implement GET /admin/export/download**

Query learning records with filters, generate CSV or JSON response as downloadable file.

CSV columns: user_id, username, poem_id, poem_title, poet_name, mastery_level, review_count, next_review_date, created_at, updated_at

**Step 3: Test manually**

Export data in both CSV and JSON formats through the admin UI.

**Step 4: Commit**

```bash
git add -A
git commit -m "feat: data export in CSV and JSON formats"
```

---

### Task 16: Integration Testing and Polish

**Files:**
- Modify: various files for bug fixes
- Create: `tests/` directory with integration tests

**Step 1: Verify full flow**

1. Start server: `cargo run`
2. Admin login → manage poets → manage poems → import JSON → view users → export data
3. API: register → login → list poems → get poem → sync progress → get due reviews

**Step 2: Fix any issues found during testing**

**Step 3: Add input validation and error display in templates**

- Flash messages for success/error feedback
- Form validation (required fields, length limits)
- Duplicate detection with user-friendly messages

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: integration testing and polish"
```

---

## Task Dependency Graph

```
Task 1 (Scaffolding)
  ├── Task 2 (Errors)
  ├── Task 3 (Migrations)
  └── Task 4 (User Model + Auth)
        ├── Task 5 (Poet/Poem Models)
        │     ├── Task 6 (Templates)
        │     │     └── Task 7 (Admin Auth)
        │     │           ├── Task 8 (Admin Poet CRUD)
        │     │           │     └── Task 9 (Admin Poem CRUD)
        │     │           │           └── Task 10 (Batch Import)
        │     │           └── Task 14 (Admin Users)
        │     └── Task 11 (API Auth)
        │           ├── Task 12 (API Poems)
        │           └── Task 13 (API Progress)
        └── Task 15 (Data Export)
              └── Task 16 (Integration + Polish)
```

Tasks 8+9, 10, 11, 14, 15 are largely independent once their prerequisites are met and can be parallelized.
