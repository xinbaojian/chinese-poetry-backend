# 古诗词学习 App 后端

古诗词学习 App 的后端服务，提供纯 JSON API，同时服务于管理后台 SPA 和 iOS 客户端。

## 技术栈

- **后端**：Rust + Axum 0.7 + SQLx 0.7 + MySQL
- **前端**：Vue 3 + TypeScript + Vite + Vue Router（管理后台 SPA）
- **认证**：JWT (HS256) + Argon2 密码哈希
- **数据库**：MySQL 8.0

## 目录结构

```
├── src/
│   ├── main.rs                 # 入口：初始化日志、加载配置、连接数据库、运行迁移、启动服务
│   ├── app.rs                  # Axum 路由注册 + 中间件 + SPA 静态文件服务
│   ├── config.rs               # TOML 配置加载
│   ├── state.rs                # AppState（Config + MySqlPool）
│   ├── errors.rs               # 统一错误处理 → JSON 响应
│   ├── auth.rs                 # 密码哈希、JWT 生成/验证、认证中间件
│   ├── models/
│   │   └── user.rs             # 用户数据模型
│   └── handlers/
│       ├── admin/              # 管理后台 API（/api/v1/admin/*）
│       │   ├── auth.rs         # 管理员登录
│       │   ├── dashboard.rs    # 仪表盘统计
│       │   ├── poems.rs        # 诗词 CRUD + 筛选
│       │   ├── poets.rs        # 诗人 CRUD
│       │   ├── users.rs        # 用户管理
│       │   ├── import.rs       # JSON 批量导入
│       │   └── export.rs       # 数据导出
│       └── api/                # iOS App API（/api/v1/*）
│           ├── auth.rs         # 注册/登录
│           ├── poems.rs        # 诗词查询
│           └── progress.rs     # 学习进度同步
├── frontend/                   # Vue 3 管理后台 SPA
│   ├── src/
│   │   ├── views/              # 页面组件
│   │   ├── router/             # 路由配置
│   │   ├── api/                # Axios 封装
│   │   ├── utils/              # 工具函数（toast、confirm）
│   │   └── styles/             # 全局样式 + 设计令牌
│   └── vite.config.ts
├── migrations/                 # 数据库迁移 SQL（启动时自动执行）
│   ├── 001_create_users.sql
│   ├── 002_create_poets.sql
│   ├── 003_create_poems.sql
│   ├── 004_create_learning_records.sql
│   ├── 005_create_review_history.sql
│   └── 006_create_settings.sql
├── config.toml.example         # 配置文件模板
└── docs/
    └── ios-api-reference.md    # iOS 客户端 API 对接文档
```

## 开发

### 前置要求

- Rust 1.70+（edition 2021）
- MySQL 8.0+
- Node.js 18+ & pnpm（或 npm）

### 1. 配置

复制配置模板并修改：

```bash
cp config.toml.example config.toml
```

编辑 `config.toml`，填入数据库连接信息和 JWT 密钥：

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "mysql://user:password@host:3306/poetry"

[auth]
jwt_secret = "your-random-secret"
jwt_exp_hours = 72
admin_username = "admin"
admin_password = "your-admin-password"
```

### 2. 启动后端

```bash
cargo run
```

服务默认监听 `127.0.0.1:3000`。首次启动会自动：
- 执行数据库迁移（创建表结构）
- 创建管理员账户（如果不存在）

### 3. 启动前端开发服务器

```bash
cd frontend
pnpm install
pnpm dev
```

前端开发服务器默认运行在 `http://localhost:5173`，API 请求通过 Vite proxy 转发到后端 `localhost:3000`。

### 4. 访问

- 管理后台：`http://localhost:5173`
- API 基础路径：`http://localhost:3000/api/v1`

## API 概览

### 管理后台 API

所有管理后台接口需要 JWT 认证（`Authorization: Bearer <token>`）且角色为 `admin`。

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/v1/admin/login | 管理员登录 |
| GET | /api/v1/admin/dashboard | 仪表盘统计 |
| GET/POST | /api/v1/admin/poems | 诗词列表 / 创建 |
| GET/PUT/DELETE | /api/v1/admin/poems/:id | 诗词详情 / 更新 / 删除 |
| GET | /api/v1/admin/poems/filter-options | 筛选选项 |
| GET/POST | /api/v1/admin/poets | 诗人列表 / 创建 |
| PUT/DELETE | /api/v1/admin/poets/:id | 诗人更新 / 删除 |
| GET/DELETE | /api/v1/admin/users | 用户列表 / 删除 |
| POST | /api/v1/admin/import | 批量导入诗词（JSON） |
| GET | /api/v1/admin/export | 导出学习数据 |

### iOS App API

App 端接口需要 JWT 认证。

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/v1/auth/register | 用户注册 |
| POST | /api/v1/auth/login | 用户登录 |
| GET | /api/v1/poems | 诗词列表（分页、筛选） |
| GET | /api/v1/poems/:id | 诗词详情 |
| GET | /api/v1/progress | 学习记录列表 |
| POST | /api/v1/progress | 同步学习进度 |
| GET | /api/v1/progress/due | 获取待复习列表 |

iOS 对接详细文档见 [docs/ios-api-reference.md](docs/ios-api-reference.md)。

## 部署

### Docker 部署（推荐）

前端编译产物通过 `rust-embed` 嵌入到 Rust 二进制中，Docker 镜像只包含一个可执行文件。

```bash
# 构建镜像
make docker-build

# 运行（挂载配置文件）
make docker-run

# 自定义镜像名和标签
make docker-build IMAGE_NAME=your-registry/poetry-backend IMAGE_TAG=v1.0.0
```

Dockerfile 使用多阶段构建：
1. **Stage 1**：Node.js 编译前端 → `frontend/dist/`
2. **Stage 2**：Rust 编译后端（`rust-embed` 将 dist 嵌入二进制）
3. **Stage 3**：精简运行镜像（debian-slim + 单个二进制）

运行时只需挂载 `config.toml`：

```bash
docker run -d \
  -p 3000:3000 \
  -v /path/to/config.toml:/app/config.toml:ro \
  chinese-poetry-backend:latest
```

### 手动构建

```bash
# 1. 构建前端
cd frontend && pnpm build && cd ..

# 2. 构建后端（自动嵌入 frontend/dist/）
cargo build --release

# 3. 运行
./target/release/chinese-poetry-backend
```

生产环境下通过环境变量指定配置文件路径：

```bash
CONFIG_PATH=/etc/chinese-poetry/config.toml ./target/release/chinese-poetry-backend
```

### 部署要点

- 前端已嵌入二进制，无需单独部署 Nginx 或静态文件服务器
- 非 API 路径自动 fallback 到 `index.html`（支持 Vue Router HTML5 history 模式）
- 数据库迁移在每次启动时自动执行（幂等）
- 建议使用 systemd 或 supervisor 管理进程
- 生产环境建议在前面加 Nginx/Caddy 做反向代理和 HTTPS 终止

### 配置示例（systemd）

```ini
[Unit]
Description=Chinese Poetry Backend
After=network.target mysql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/chinese-poetry-backend
ExecStart=/opt/chinese-poetry-backend/chinese-poetry-backend
Environment=CONFIG_PATH=/opt/chinese-poetry-backend/config.toml
Restart=always

[Install]
WantedBy=multi-user.target
```
