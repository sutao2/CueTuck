# 变更提案：预发后端持久化

| 字段 | 值 |
|---|---|
| 状态 | 接受 |
| 日期 | 2026-08-24 |
| 影响规格 | [auth](../../specs/auth/spec.md) · [square](../../specs/square/spec.md) · [publish](../../specs/publish/spec.md) · [admin](../../specs/admin/spec.md) |

2026-08-24 人要求完成整个后端，复用本机已启动的 Postgres / Redis / MinIO，并复用 Google / GitHub 授权码流程；表不对时可删重建。视为接受。决定以 [ADR 0012](../../architecture/decisions/0012-postgres-backend.md) 与 [ADR 0013](../../architecture/decisions/0013-oauth-google-github.md) 为准。

## 为什么

预发 API 在内存里，重启丢失。旧 Spring 库 `pl` 不能当现行合同。本机基础设施已经起来，应接到本仓库 `/v1/`。

## 做什么

1. 独立库 `promptark`：账号、令牌、广场、投稿、收藏、设置、OAuth 绑定、媒体元数据。
2. Argon2id 口令。
3. Google / GitHub 授权码登录（旧环境变量可复用）。
4. Redis 用于 Refresh 与 OAuth 浏览器会话。
5. MinIO 用于登录后上传对象。
6. 表不对或缺字段时删除重建。

## 不做什么

- 云同步个人库、Stripe、商店上架、QQ / LinuxDo
- 旧 `/api/v1` 路径
- 声称生产托管或覆盖率已达 80%/70%（未实测不得写）
- 改启动器或 MCP 去请求广场

## 依赖

M9 已关闭。执行 [2026-08-24-postgres-backend.md](../../plans/2026-08-24-postgres-backend.md)。
