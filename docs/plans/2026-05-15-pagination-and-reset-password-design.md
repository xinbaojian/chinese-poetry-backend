# 分页调整 + 重置密码功能设计

## 需求

1. 诗词管理、诗人管理分页列表默认每页 10 条，用户可手动调整
2. 用户管理新增重置密码功能（重置为固定默认密码 123456）

## 设计

### 1. 分页调整

**后端改动（3 个 handler）：**

- `src/handlers/admin/poems.rs`：`per_page` 默认值 20→10，新增 `per_page` 查询参数
- `src/handlers/admin/poets.rs`：同上
- `src/handlers/admin/users.rs`：同上

改动方式：在 handler 的查询参数结构中增加 `per_page: Option<u32>`，取值时用 `.unwrap_or(10)`，并限制最大值（如 100）。

**前端改动（3 个页面）：**

- `Poems.vue`、`Poets.vue`、`Users.vue`：
  - API 请求传入 `per_page` 参数
  - 在分页组件旁增加每页条数选择器（10/20/50）
  - 切换时重新请求第一页

### 2. 重置密码

**后端：**

- `src/handlers/admin/users.rs`：新增 `reset_password` handler
- `src/app.rs`：注册路由 `PUT /api/v1/admin/users/:id/reset-password`
- 逻辑：查找用户 → 用 Argon2 哈希默认密码 `123456` → UPDATE 数据库

**前端：**

- `Users.vue`：操作列增加「重置密码」按钮 → confirm 弹窗 → 调用 API → toast 提示
