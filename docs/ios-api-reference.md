# iOS 客户端 API 对接文档

> 基础地址：`http(s)://<host>:3000/api/v1`
>
> 所有接口统一返回 JSON。认证接口无需 Token，其余接口需在 Header 携带 `Authorization: Bearer <token>`。

---

## 目录

1. [通用约定](#1-通用约定)
2. [认证模块](#2-认证模块)
3. [诗词模块](#3-诗词模块)
4. [学习进度模块](#4-学习进度模块)
5. [错误码参考](#5-错误码参考)
6. [Swift 数据模型参考](#6-swift-数据模型参考)

---

## 1. 通用约定

### 请求格式

```
Content-Type: application/json
Authorization: Bearer <jwt_token>   // 需要认证的接口
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

### JWT Token

- 算法：HS256
- 有效期：72 小时（可配置）
- Payload：

```json
{
  "sub": "用户ID（字符串）",
  "username": "用户名",
  "role": "user 或 admin",
  "iat": 1715664000,
  "exp": 1715923200
}
```

- Token 过期后需重新调用登录接口获取新 Token

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
  "user": {
    "id": 5,
    "username": "zhangsan",
    "role": "user"
  }
}
```

错误场景：
- `401` — 用户名或密码错误

> **对接建议**：注册/登录成功后，将 `token` 存入 Keychain，`user` 信息存入 UserDefaults 或本地数据库。后续所有请求从 Keychain 读取 Token。

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

## 5. 错误码参考

| HTTP 状态码 | 含义 | 典型场景 |
|------------|------|---------|
| `200` | 成功 | — |
| `400` | 参数错误 | 用户名长度不符、密码太短、必填字段为空 |
| `401` | 未认证 / 认证失败 | Token 缺失、Token 过期、用户名密码错误 |
| `404` | 资源不存在 | 诗词 ID 不存在 |
| `409` | 冲突 | 用户名已注册 |
| `500` | 服务器内部错误 | 数据库异常 |

> **对接建议**：收到 `401` 时应清除本地 Token 并跳转到登录页。其他错误展示 `error` 字段的中文信息即可。

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
    let token: String
    let user: UserInfo
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

    var token: String?  // 从 Keychain 读取

    func request<T: Decodable>(
        _ path: String,
        method: String = "GET",
        body: Encodable? = nil
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

        if http.statusCode == 401 {
            // Token 过期，跳转登录
            throw APIError.unauthorized
        }

        if !(200...299).contains(http.statusCode) {
            let err = try? JSONDecoder().decode(ErrorResponse.self, from: data)
            throw APIError.serverError(err?.error ?? "未知错误", http.statusCode)
        }

        return try JSONDecoder().decode(T.self, from: data)
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

// 获取诗词列表
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
```

---

## 附录：典型数据流

```
┌─────────┐                          ┌─────────┐
│  iOS App │                          │  Server  │
└────┬─────┘                          └────┬─────┘
     │                                     │
     │  POST /auth/register                │
     │  { username, password }             │
     │────────────────────────────────────►│
     │  { token, user }                    │
     │◄────────────────────────────────────│
     │                                     │
     │  GET /poems?grade=3                 │  （带 Bearer Token）
     │────────────────────────────────────►│
     │  { poems, total, page }             │
     │◄────────────────────────────────────│
     │                                     │
     │  POST /progress                     │  （学习完成后同步）
     │  { records: [...] }                 │
     │────────────────────────────────────►│
     │  { synced, skipped }                │
     │◄────────────────────────────────────│
     │                                     │
     │  GET /progress/due                  │  （检查待复习）
     │────────────────────────────────────►│
     │  [ learning_records... ]            │
     │◄────────────────────────────────────│
```
