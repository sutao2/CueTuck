# 设计：M7 合同补齐

对照 [提案](proposal.md)，不重复「为什么」。提案未接受前本文不是现行真相。

## 方案

- 收藏：同一 `backend/` 内存表，键为邮箱 + 广场条目 id。桌面已登录走 `PUT`/`DELETE`，未登录仍只开登录。
- 轮换：`POST /v1/session/refresh` 或等价路径若合同已有则实现；否则在 `square.yaml` 补一条刷新 path，接受提案后进 INDEX。旧令牌作废。
- `GET /v1/admin/me`：复用现有角色检查，返回 `{ email, role }`。

## 文件

| 路径 | 职责 |
|---|---|
| `docs/reference/openapi/square.yaml` | 确认收藏与刷新 path |
| `backend/src` | favorites、轮换、`/v1/admin/me` |
| `desktop/` | 已登录收藏；Refresh 轮换进钥匙串 |
| `admin-web/` | 可用 `/v1/admin/me` 校验角色 |

## 风险

- 刷新 path 若合同没有，须在 Task 0 后先改 OpenAPI，再写失败测试。
- 预发内存存储重启即丢收藏与会话，不得写成已持久化账号系统。
