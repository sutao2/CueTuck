# 变更提案：M7 合同补齐

| 字段 | 值 |
|---|---|
| 状态 | 草稿 |
| 日期 | 2026-08-23 |
| 影响规格 | [square](../../specs/square/spec.md) · [auth](../../specs/auth/spec.md) · [admin](../../specs/admin/spec.md) |

未接受。Agent 不得自称接受。接受前不得实现收藏写路径、令牌轮换或 `GET /v1/admin/me`。

## 为什么

M5 / M6 已接受的 OpenAPI 与规格要求：已登录可收藏/取消收藏、Access 与 Refresh 轮换、管理端能读当前管理员身份。实现只做到了未登录收藏门闩、签发令牌、审核与用户列表。合同比运行时多出三截。

本仓库 `backend/` 仍是预发，不声称生产。不借此打开 OAuth、计费或商店分发。

## 做什么

1. **已登录收藏。** `PUT` / `DELETE` `/v1/favorites/{id}` 与列表生效。收藏是账号关系，不写本地 `source=downloaded` 副本。未登录仍只打开登录。
2. **令牌轮换。** 用 Refresh 换发后，旧 Access 与旧 Refresh 均失效。新 Refresh 只进系统钥匙串，不进 Web Storage。管理端仍不持久化 Refresh。
3. **管理员身份。** `GET /v1/admin/me` 对管理员返回邮箱与角色；普通 Access 拒绝。

## 不做什么

- 密码 KDF、Postgres、OAuth、计费、完整 Web 个人库
- 把管理页打进桌面或让启动器请求广场 / 管理
- 声称生产已上线、商店分发或公开下载
- 在提案被接受前写上述 API 或桌面收藏写路径

## 依赖

M6 已完成。本提案被接受后，才执行 [M7 计划](../../plans/2026-08-23-m7-contract-gaps.md)。
