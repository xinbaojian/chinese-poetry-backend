# iOS 客户端 API 对接文档

> 基础地址：`http(s)://<host>:3000/api/v1`
>
> 所有接口统一返回 JSON。认证接口无需 Token，其余接口需在 Header 携带 `Authorization: Bearer <access_token>`。

---

## 目录

1. [通用约定](#1-通用约定)
2. [认证模块](#2-认证模块)
3. [诗词模块](#3-诗词模块)
4. [学习进度模块](#4-学习进度模块)
5. [错误码参考](#5-错误码参考)
6. [Swift 数据模型参考](#6-swift-数据模型参考)
7. [令牌刷新流程](#7-令牌刷新流程)

---

## 1. 通用约定

### 请求格式

```
Content-Type: application/json
Authorization: Bearer <access_token>   // 需要认证的接口
```

### 成功响应

各接口独立定义响应体，均为 JSON。

### 错误响应

所有错误统一格式：

```json
{
  "error": "错误描述（中文）"
}
```

### 双令牌机制

系统使用双令牌认证，登录/注册成功后返回两个令牌：

| 令牌 | 类型 | 有效期 | 用途 |
|------|------|--------|------|
| `token`（access_token） | JWT (HS256) | 2 小时 | 放在 `Authorization` 头中访问受保护接口 |
| `refresh_token` | 不透明随机字符串 | 30 天 | access_token 过期后用它换新令牌对 |

**access_token** Payload：

```json
{
  "sub": "用户ID（字符串）",
  "username": "用户名",
  "role": "user 或 admin",
  "iat": 1715664000,
  "exp": 1715671200
}
```

**核心规则：**
- access_token 过期后，用 refresh_token 调用刷新接口获取新的令牌对
- 每次刷新后，旧 refresh_token 立即失效（令牌轮换）
- 修改密码后，该用户所有 refresh_token 失效
- refresh_token 过期或已吊销，需重新登录

> **存储建议**：`token` 和 `refresh_token` 均存入 Keychain，不要存 UserDefaults。

---

## 2. 认证模块

### 2.1 注册

```
POST /api/v1/auth/register
```

**无需认证**

请求体：

```json
{
  "username": "zhangsan",
  "password": "123456"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| username | String | ✅ | 3-64 个字符 |
| password | String | ✅ | 至少 6 个字符 |

成功响应 `200`：

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "user": {
    "id": 5,
    "username": "zhangsan",
    "role": "user"
  }
}
```

错误场景：
- `400` — 用户名长度不符 / 密码长度不足
- `409` — 用户名已存在

---

### 2.2 登录

```
POST /api/v1/auth/login
```

**无需认证**

请求体：

```json
{
  "username": "zhangsan",
  "password": "123456"
}
```

成功响应 `200`：

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "user": {
    "id": 5,
    "username": "zhangsan",
    "role": "user"
  }
}
```

错误场景：
- `401` — 用户名或密码错误

---

### 2.3 刷新令牌

```
POST /api/v1/auth/refresh
```

**无需认证**

请求体：

```json
{
  "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

成功响应 `200`：

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...(新 access_token)",
  "refresh_token": "f9e8d7c6-b5a4-3210-fedc-ba0987654321",
  "user": {
    "id": 5,
    "username": "zhangsan",
    "role": "user"
  }
}
```

> **重要**：每次刷新成功后，旧 refresh_token 立即失效。必须用新返回的 `token` 和 `refresh_token` 替换本地存储。

错误场景：
- `401` — refresh_token 过期、已吊销或无效（需重新登录）

---

### 2.4 退出登录

```
POST /api/v1/auth/logout
```

**无需认证**（通过请求体中的 refresh_token 识别会话）

请求体：

```json
{
  "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

成功响应 `200`：

```json
{
  "message": "已退出登录"
}
```

> 调用后应同时清除本地存储的 token 和 refresh_token。

---

### 2.5 修改密码

```
PUT /api/v1/auth/password
```

**需要认证**

请求体：

```json
{
  "old_password": "123456",
  "new_password": "654321"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| old_password | String | ✅ | 当前密码 |
| new_password | String | ✅ | 新密码，至少 6 个字符 |

成功响应 `200`：

```json
{
  "message": "密码修改成功"
}
```

> **注意**：修改密码后，该用户所有 refresh_token 被吊销。客户端应清除本地令牌并引导用户重新登录。

错误场景：
- `400` — 新密码长度不足
- `401` — 旧密码错误

---

## 3. 诗词模块

### 3.1 获取诗词列表

```
GET /api/v1/poems
```

**需要认证**

查询参数：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| keyword | String | ❌ | 按标题模糊搜索 |
| dynasty | String | ❌ | 按朝代精确筛选（如 "唐"、"宋"） |
| category | String | ❌ | 按分类精确筛选（如 "五言绝句"） |
| grade | String | ❌ | 按年级筛选（1-6 对应小学年级） |
| page | Int | ❌ | 页码，默认 1 |
| per_page | Int | ❌ | 每页条数，默认 20，最大 100 |

示例请求：

```
GET /api/v1/poems?dynasty=唐&grade=3&page=1&per_page=10
```

成功响应 `200`：

```json
{
  "poems": [
    {
      "id": 1,
      "title": "静夜思",
      "poet_name": "李白",
      "dynasty": "唐",
      "category": "五言绝句",
      "grade": 3,
      "content": "[\"床前明月光\",\"疑是地上霜\",\"举头望明月\",\"低头思故乡\"]",
      "translation": "明亮的月光洒在窗户纸上…"
    }
  ],
  "total": 128,
  "page": 1,
  "per_page": 10
}
```

> **注意**：`content` 字段是 **JSON 字符串**（而非数组），iOS 端需要二次解析：
> ```swift
> let paragraphs = try JSONDecoder().decode([String].self, from: poem.content.data(using: .utf8)!)
> ```
>
> `translation` 为可空字段，不存在时返回 `null`。

---

### 3.2 获取诗词详情

```
GET /api/v1/poems/{id}
```

**需要认证**

路径参数：

| 参数 | 类型 | 说明 |
|------|------|------|
| id | Int | 诗词 ID |

成功响应 `200`：

```json
{
  "id": 1,
  "title": "静夜思",
  "poet_name": "李白",
  "dynasty": "唐",
  "category": "五言绝句",
  "grade": 3,
  "content": "[\"床前明月光\",\"疑是地上霜\",\"举头望明月\",\"低头思故乡\"]",
  "translation": "明亮的月光洒在窗户纸上…"
}
```

错误场景：
- `404` — 诗词不存在

---

## 4. 学习进度模块

### 4.1 获取全部学习记录

```
GET /api/v1/progress
```

**需要认证**

成功响应 `200`：

```json
[
  {
    "id": 1,
    "poem_id": 5,
    "poem_title": "春晓",
    "poet_name": "孟浩然",
    "mastery_level": "learning",
    "review_count": 3,
    "next_review_date": "2025-06-01T08:00:00",
    "created_at": "2025-05-20T10:30:00",
    "updated_at": "2025-05-25T14:20:00"
  }
]
```

| 字段 | 类型 | 说明 |
|------|------|------|
| id | Int | 学习记录 ID |
| poem_id | Int | 关联的诗词 ID |
| poem_title | String | 诗词标题（冗余，方便展示） |
| poet_name | String | 诗人姓名（冗余） |
| mastery_level | String | 掌握程度：`new` / `learning` / `reviewing` / `mastered` |
| review_count | Int | 已复习次数 |
| next_review_date | String? | 下次复习时间（ISO 格式），可能为 null |
| created_at | String | 首次学习时间 |
| updated_at | String | 最后更新时间 |

> **对接建议**：App 启动或进入学习页面时调用，拉取全量记录后覆盖本地数据库，用于离线展示。

---

### 4.2 同步学习进度

```
POST /api/v1/progress
```

**需要认证**

请求体：

```json
{
  "records": [
    {
      "poem_id": 5,
      "mastery_level": "reviewing",
      "review_count": 4,
      "next_review_date": "2025-06-15T08:00:00",
      "updated_at": "2025-05-28T09:00:00"
    },
    {
      "poem_id": 12,
      "mastery_level": "new",
      "review_count": 0,
      "next_review_date": null,
      "updated_at": "2025-05-28T09:05:00"
    }
  ]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| records | Array | ✅ | 要同步的记录数组 |
| records[].poem_id | Int | ✅ | 诗词 ID |
| records[].mastery_level | String | ✅ | 掌握程度 |
| records[].review_count | Int | ✅ | 复习次数 |
| records[].next_review_date | String? | ❌ | 下次复习时间，传 null 表示无计划 |
| records[].updated_at | String | ✅ | 客户端记录的最后更新时间 |

成功响应 `200`：

```json
{
  "synced": 2,
  "skipped": 0
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| synced | Int | 成功同步（新建或更新）的记录数 |
| skipped | Int | 跳过的记录数（服务端数据更新时间 ≥ 客户端） |

> **冲突策略**：基于 `updated_at` 时间戳，服务端保留**更新时间更晚**的一方。如果客户端数据比服务端旧，该条记录会被跳过。

---

### 4.3 获取待复习列表

```
GET /api/v1/progress/due
```

**需要认证**

成功响应 `200`：

```json
[
  {
    "id": 3,
    "poem_id": 8,
    "poem_title": "登鹳雀楼",
    "poet_name": "王之涣",
    "mastery_level": "reviewing",
    "review_count": 5,
    "next_review_date": "2025-05-27T08:00:00",
    "created_at": "2025-05-10T10:00:00",
    "updated_at": "2025-05-27T07:00:00"
  }
]
```

响应格式同 [4.1 获取全部学习记录](#41-获取全部学习记录)，仅返回 `next_review_date <= 当前时间` 的记录。

> **对接建议**：用于首页「今日待复习」入口，调用后展示数量角标。用户点击后加载待复习诗词列表。

---

### 4.4 删除学习记录

```
DELETE /api/v1/progress/{poem_id}
```

**需要认证**

路径参数：

| 参数 | 类型 | 说明 |
|------|------|------|
| poem_id | Int | 诗词 ID |

成功响应 `200`：

```json
{
  "message": "删除成功"
}
```

---

## 5. 错误码参考

| HTTP 状态码 | 含义 | 典型场景 |
|------------|------|---------|
| `200` | 成功 | — |
| `400` | 参数错误 | 用户名长度不符、密码太短、必填字段为空 |
| `401` | 未认证 / 认证失败 | Token 缺失、Token 过期、refresh_token 无效或过期 |
| `404` | 资源不存在 | 诗词 ID 不存在 |
| `409` | 冲突 | 用户名已注册 |
| `500` | 服务器内部错误 | 数据库异常 |

> **对接建议**：收到 `401` 时，**先尝试用 refresh_token 刷新令牌**（见[第 7 节](#7-令牌刷新流程)），刷新失败再清除本地数据并跳转登录页。

---

## 6. Swift 数据模型参考

```swift
import Foundation

// MARK: - Auth

struct LoginRequest: Encodable {
    let username: String
    let password: String
}

struct RegisterRequest: Encodable {
    let username: String
    let password: String
}

struct AuthResponse: Decodable {
    let token: String              // access_token（JWT，2 小时有效）
    let refresh_token: String      // refresh_token（30 天有效）
    let user: UserInfo
}

struct RefreshRequest: Encodable {
    let refresh_token: String
}

struct UserInfo: Decodable {
    let id: UInt64
    let username: String
    let role: String
}

// MARK: - Poem

struct PoemListResponse: Decodable {
    let poems: [PoemItem]
    let total: Int64
    let page: UInt32
    let per_page: UInt32
}

struct PoemItem: Decodable {
    let id: UInt64
    let title: String
    let poet_name: String
    let dynasty: String
    let category: String
    let grade: UInt8
    let content: String       // JSON 字符串，需二次解析为 [String]
    let translation: String?
}

// content 解析扩展
extension PoemItem {
    var paragraphs: [String] {
        guard let data = content.data(using: .utf8),
              let lines = try? JSONDecoder().decode([String].self, from: data) else {
            return [content]
        }
        return lines
    }
}

// MARK: - Progress

struct LearningRecord: Decodable {
    let id: UInt64
    let poem_id: UInt64
    let poem_title: String
    let poet_name: String
    let mastery_level: String   // new / learning / reviewing / mastered
    let review_count: UInt32
    let next_review_date: String?
    let created_at: String
    let updated_at: String
}

struct SyncRequest: Encodable {
    let records: [SyncRecord]
}

struct SyncRecord: Encodable {
    let poem_id: UInt64
    let mastery_level: String
    let review_count: UInt32
    let next_review_date: String?
    let updated_at: String
}

struct SyncResponse: Decodable {
    let synced: UInt32
    let skipped: UInt32
}

// MARK: - Error

struct ErrorResponse: Decodable {
    let error: String
}
```

### 网络层示例（URLSession）

```swift
class APIClient {
    static let shared = APIClient()
    private let baseURL = "http://<host>:3000/api/v1"

    // 存入 Keychain
    var token: String?
    var refreshToken: String?

    /// 发起请求，401 时自动尝试刷新令牌后重试一次
    func request<T: Decodable>(
        _ path: String,
        method: String = "GET",
        body: Encodable? = nil,
        retry: Bool = true
    ) async throws -> T {
        var request = URLRequest(url: URL(string: "\(baseURL)\(path)")!)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")

        if let token {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        if let body {
            request.httpBody = try JSONEncoder().encode(body)
        }

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }

        // 401 且有 refresh_token：尝试刷新后重试
        if http.statusCode == 401 && retry {
            if let refreshed = try? await refreshTokens() {
                self.token = refreshed.token
                self.refreshToken = refreshed.refresh_token
                // 用新令牌重试原请求（不再重试）
                return try await request(path, method: method, body: body, retry: false)
            } else {
                // 刷新也失败，需重新登录
                throw APIError.unauthorized
            }
        }

        if !(200...299).contains(http.statusCode) {
            let err = try? JSONDecoder().decode(ErrorResponse.self, from: data)
            throw APIError.serverError(err?.error ?? "未知错误", http.statusCode)
        }

        return try JSONDecoder().decode(T.self, from: data)
    }

    /// 刷新令牌
    private func refreshTokens() async throws -> AuthResponse {
        guard let refreshToken else { throw APIError.unauthorized }

        var request = URLRequest(url: URL(string: "\(baseURL)/auth/refresh")!)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(RefreshRequest(refresh_token: refreshToken))

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let http = response as? HTTPURLResponse,
              (200...299).contains(http.statusCode) else {
            throw APIError.unauthorized
        }

        return try JSONDecoder().decode(AuthResponse.self, from: data)
    }

    /// 退出登录
    func logout() async throws {
        guard let refreshToken else { return }
        let _: ErrorResponse? = try? await request(
            "/auth/logout",
            method: "POST",
            body: RefreshRequest(refresh_token: refreshToken),
            retry: false
        )
        self.token = nil
        self.refreshToken = nil
    }
}

enum APIError: LocalizedError {
    case invalidResponse
    case unauthorized
    case serverError(String, Int)

    var errorDescription: String? {
        switch self {
        case .unauthorized: return "登录已过期，请重新登录"
        case .serverError(let msg, _): return msg
        case .invalidResponse: return "网络响应异常"
        }
    }
}
```

### 调用示例

```swift
// 注册
let auth: AuthResponse = try await APIClient.shared.request(
    "/auth/register",
    method: "POST",
    body: RegisterRequest(username: "test", password: "123456")
)
APIClient.shared.token = auth.token
APIClient.shared.refreshToken = auth.refresh_token

// 获取诗词列表（token 过期会自动刷新）
let list: PoemListResponse = try await APIClient.shared.request(
    "/poems?dynasty=唐&page=1"
)

// 同步进度
let sync: SyncResponse = try await APIClient.shared.request(
    "/progress",
    method: "POST",
    body: SyncRequest(records: [
        SyncRecord(
            poem_id: 1,
            mastery_level: "reviewing",
            review_count: 3,
            next_review_date: "2025-06-01T08:00:00",
            updated_at: "2025-05-28T09:00:00"
        )
    ])
)

// 获取待复习
let due: [LearningRecord] = try await APIClient.shared.request("/progress/due")

// 退出登录
try await APIClient.shared.logout()
```

---

## 7. 令牌刷新流程

```
┌──────────┐                                   ┌──────────┐
│  iOS App │                                   │  Server  │
└─────┬────┘                                   └─────┬────┘
      │                                              │
      │  ① 请求受保护接口（带 access_token）          │
      │─────────────────────────────────────────────►│
      │                                              │
      │       ② 返回 401（access_token 过期）         │
      │◄─────────────────────────────────────────────│
      │                                              │
      │  ③ POST /auth/refresh                        │
      │  { refresh_token }                           │
      │─────────────────────────────────────────────►│
      │                                              │
      │       ④ 返回新令牌对                          │
      │  { token, refresh_token, user }              │
      │◄─────────────────────────────────────────────│
      │                                              │
      │  ⑤ 用新 access_token 重试原请求               │
      │─────────────────────────────────────────────►│
      │                                              │
      │       ⑥ 返回业务数据                          │
      │◄─────────────────────────────────────────────│
      │                                              │
```

**刷新失败场景**（步骤 ④ 返回 401）：

- refresh_token 过期（超过 30 天未使用）
- refresh_token 已被轮换（被其他设备使用过）
- 用户已修改密码
- 用户已登出

此时应清除本地所有令牌数据，跳转到登录页。

> **并发请求处理**：多个请求同时收到 401 时，应只发起一次刷新，其余请求等待刷新完成后用新令牌重试。可用 `NSLock` 或 actor 保护刷新逻辑。

---

## 附录：完整数据流

```
┌──────────┐                                   ┌──────────┐
│  iOS App │                                   │  Server  │
└─────┬────┘                                   └─────┬────┘
      │                                              │
      │  POST /auth/register                         │
      │  { username, password }                      │
      │─────────────────────────────────────────────►│
      │  { token, refresh_token, user }              │
      │◄─────────────────────────────────────────────│
      │                                              │
      │  GET /poems?grade=3  （Bearer access_token）  │
      │─────────────────────────────────────────────►│
      │  { poems, total, page }                      │
      │◄─────────────────────────────────────────────│
      │                                              │
      │  POST /progress  （学习完成后同步）             │
      │  { records: [...] }                          │
      │─────────────────────────────────────────────►│
      │  { synced, skipped }                         │
      │◄─────────────────────────────────────────────│
      │                                              │
      │  GET /progress/due  （检查待复习）              │
      │─────────────────────────────────────────────►│
      │  [ learning_records... ]                     │
      │◄─────────────────────────────────────────────│
      │                                              │
      │  POST /auth/logout  （退出登录）               │
      │  { refresh_token }                           │
      │─────────────────────────────────────────────►│
      │  { message }                                 │
      │◄─────────────────────────────────────────────│
      │                                              │
```
