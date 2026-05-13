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
