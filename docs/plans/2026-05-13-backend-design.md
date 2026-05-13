# Chinese Poetry Backend - Design Document

## Overview

古诗词学习 App 的后端管理平台，基于 Axum 0.7 + MySQL + Askama 模板引擎。

服务同时提供两套接口：
- **管理后台**：Askama 服务端渲染 HTML，管理员通过浏览器操作
- **App API**：REST JSON 接口，供 iOS App 用户注册登录、同步学习进度

## Architecture

```
┌─────────────┐     ┌──────────────────────────────────────┐
│   iOS App   │────▶│  REST API (JSON)                     │
│             │◀────│  /api/v1/auth/*   /api/v1/poems/*    │
└─────────────┘     │  /api/v1/progress/*                  │
                    │                                      │
┌─────────────┐     │  Admin Web (Askama SSR)              │
│   管理员     │────▶│  /admin/login                        │
│   浏览器     │◀────│  /admin/poems  /admin/users  etc.   │
└─────────────┘     └──────────┬───────────────────────────┘
                               │
                        ┌──────▼──────┐
                        │   MySQL     │
                        └─────────────┘
```

## Tech Stack

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | Edition 2021 |
| Web Framework | Axum | 0.7 |
| Database | MySQL | 8.0+ |
| Async DB Driver | SQLx | 0.7 |
| Template Engine | Askama | 0.12 |
| Password Hash | Argon2 | - |
| JWT | jsonwebtoken | 9 |
| Session | tower-cookies | - |
| Logging | tracing + tracing-subscriber | - |
| Config | config crate (TOML) | - |

## Database Schema

### users

```sql
CREATE TABLE users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(64) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role ENUM('admin', 'user') NOT NULL DEFAULT 'user',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### poets

```sql
CREATE TABLE poets (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    dynasty VARCHAR(32) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_name_dynasty (name, dynasty)
);
```

### poems

```sql
CREATE TABLE poems (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    poet_id BIGINT UNSIGNED NOT NULL,
    dynasty VARCHAR(32) NOT NULL,
    category VARCHAR(64) NOT NULL DEFAULT '',
    grade TINYINT UNSIGNED NOT NULL DEFAULT 0,
    content JSON NOT NULL,
    translation TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_category (category),
    INDEX idx_grade (grade),
    INDEX idx_dynasty (dynasty),
    FOREIGN KEY (poet_id) REFERENCES poets(id)
);
```

### learning_records

```sql
CREATE TABLE learning_records (
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

### review_history

```sql
CREATE TABLE review_history (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    learning_record_id BIGINT UNSIGNED NOT NULL,
    reviewed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    mastery_level ENUM('proficient', 'fair', 'weak') NOT NULL,
    FOREIGN KEY (learning_record_id) REFERENCES learning_records(id) ON DELETE CASCADE
);
```

### settings

```sql
CREATE TABLE settings (
    `key` VARCHAR(128) NOT NULL PRIMARY KEY,
    `value` VARCHAR(512) NOT NULL
);
```

## Project Structure

```
chinese-poetry-backend/
├── Cargo.toml
├── config.toml                    # 应用配置
├── migrations/                    # SQL 迁移文件
│   ├── 001_create_users.sql
│   ├── 002_create_poets.sql
│   ├── 003_create_poems.sql
│   ├── 004_create_learning_records.sql
│   ├── 005_create_review_history.sql
│   └── 006_create_settings.sql
├── static/                        # CSS/JS 静态资源
│   └── css/
│       └── style.css
├── templates/                     # Askama HTML 模板
│   ├── base.html                  # 布局骨架
│   ├── login.html
│   ├── admin/
│   │   ├── dashboard.html
│   │   ├── poems.html             # 诗词列表
│   │   ├── poem_form.html         # 新增/编辑诗词
│   │   ├── poets.html             # 诗人列表
│   │   ├── poet_form.html
│   │   ├── users.html             # 用户列表
│   │   ├── import.html            # 批量导入
│   │   └── export.html            # 数据导出
│   └── partials/
│       ├── pagination.html
│       ├── poem_card.html
│       └── flash_message.html
└── src/
    ├── main.rs                    # 入口
    ├── config.rs                  # 配置加载
    ├── app.rs                     # Axum Router 组装
    ├── db.rs                      # 数据库连接池
    ├── models/
    │   ├── mod.rs
    │   ├── user.rs
    │   ├── poet.rs
    │   ├── poem.rs
    │   ├── learning_record.rs
    │   └── review_history.rs
    ├── handlers/
    │   ├── mod.rs
    │   ├── admin/
    │   │   ├── mod.rs
    │   │   ├── auth.rs            # 管理员登录/登出
    │   │   ├── dashboard.rs
    │   │   ├── poems.rs           # 诗词 CRUD 页面
    │   │   ├── poets.rs           # 诗人 CRUD 页面
    │   │   ├── users.rs           # 用户管理
    │   │   ├── import.rs          # 批量导入
    │   │   └── export.rs          # 数据导出
    │   └── api/
    │       ├── mod.rs
    │       ├── auth.rs            # App 用户注册/登录
    │       ├── poems.rs           # 诗词查询 API
    │       └── progress.rs        # 学习进度同步 API
    ├── services/
    │   ├── mod.rs
    │   ├── user_service.rs
    │   ├── poem_service.rs
    │   ├── poet_service.rs
    │   ├── learning_service.rs
    │   ├── import_service.rs
    │   └── export_service.rs
    ├── middleware/
    │   ├── mod.rs
    │   ├── auth.rs                # 认证中间件（Session + JWT）
    │   └── logging.rs
    └── errors.rs                  # 统一错误处理
```

## API Design

### Admin Routes (Askama SSR, Session Auth)

| Method | Path | Description |
|--------|------|-------------|
| GET/POST | /admin/login | 管理员登录页 |
| POST | /admin/logout | 登出 |
| GET | /admin/ | 仪表盘首页 |
| GET | /admin/poems | 诗词列表（分页、搜索、筛选） |
| GET/POST | /admin/poems/new | 新增诗词 |
| GET/POST | /admin/poems/{id}/edit | 编辑诗词 |
| POST | /admin/poems/{id}/delete | 删除诗词 |
| GET | /admin/poets | 诗人列表 |
| GET/POST | /admin/poets/new | 新增诗人 |
| GET/POST | /admin/poets/{id}/edit | 编辑诗人 |
| POST | /admin/poets/{id}/delete | 删除诗人 |
| GET | /admin/users | 用户列表 |
| GET/POST | /admin/import | 批量导入诗词（JSON 文件上传） |
| GET | /admin/export | 数据导出（CSV/JSON） |

### App API Routes (JSON, JWT Auth)

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/auth/register | 用户注册 |
| POST | /api/v1/auth/login | 用户登录，返回 JWT |
| GET | /api/v1/poems | 诗词列表（分页、筛选） |
| GET | /api/v1/poems/{id} | 诗词详情 |
| GET | /api/v1/progress | 获取当前用户所有学习记录 |
| POST | /api/v1/progress | 上传/同步学习记录 |
| GET | /api/v1/progress/due | 获取今日待复习诗词 |

## Module Details

### 1. User System

- **管理员**：通过 /admin/login 页面登录，Session + Cookie 认证
- **App 用户**：通过 /api/v1/auth/register 注册，/api/v1/auth/login 登录获取 JWT
- 密码统一使用 Argon2 哈希
- 初始化时通过 seed 脚本创建默认管理员账号

### 2. Poem Management

- 管理员可在后台 CRUD 诗词和诗人
- 支持按朝代、分类、年级、标题搜索筛选
- 诗词内容以 JSON 数组存储（对应 App 的 paragraphs 字段）

### 3. Batch Import

- 管理员上传 JSON 文件批量导入诗词
- 支持的 JSON 格式（兼容现有数据源）：
  ```json
  [{"title": "...", "author": "...", "paragraphes": ["..."], "id": "..."}]
  ```
- 自动去重（按标题 + 作者匹配）
- 导入结果反馈：成功数、跳过数、失败数

### 4. Learning Progress Sync

- App 端通过 API 上传学习记录
- 服务端存储 learning_records 和 review_history
- 同步逻辑：以最新 updated_at 为准，冲突时取较新的版本
- 提供获取全量学习记录和获取待复习诗词的接口

### 5. Data Export

- 管理员可导出用户学习数据
- 支持 CSV 和 JSON 两种格式
- 可按用户、时间范围筛选

## Scope (Phase 1 vs Future)

| Feature | Phase 1 (Now) | Future |
|---------|:---:|:---:|
| User auth (admin + app) | ✓ | |
| Poem CRUD + batch import | ✓ | |
| Learning progress sync | ✓ | |
| Data export | ✓ | |
| Statistics dashboard | | ✓ |
| Announcement/notification | | ✓ |
| Rate limiting | | ✓ |
| Email verification | | ✓ |
