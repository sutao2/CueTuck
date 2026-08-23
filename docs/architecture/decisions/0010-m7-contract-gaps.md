# 10. M7 补齐已登录收藏、令牌轮换与管理员身份

- Status: accepted
- Date: 2026-08-23
- 关联：[ADR 0008](0008-m5-backend-contract.md) · [ADR 0009](0009-m6-admin-console.md) · [广场](../../specs/square/spec.md) · [认证](../../specs/auth/spec.md) · [管理台](../../specs/admin/spec.md) · [提案](../../changes/m7-contract-gaps/proposal.md)

## Context

M5 / M6 已接受的 OpenAPI 与规格要求已登录收藏、Access/Refresh 轮换、以及 `GET /v1/admin/me`。实现只做到未登录收藏门闩、签发令牌、审核与用户列表。人于 2026-08-23 授权通宵继续后续开发，视为接受本提案。本仓库 `backend/` 仍是预发。

## Decision

在同一 Axum `backend/` 补齐这三截：收藏是账号关系（邮箱 + 广场条目 id），不写作者本地 `source=downloaded` 副本；`POST /v1/session/refresh` 轮换后旧 Access 与旧 Refresh 均失效，新 Refresh 只进系统钥匙串；`GET /v1/admin/me` 对管理员返回邮箱与角色，普通 Access 拒绝。不接 OAuth，不把管理页打进桌面，启动器不请求广场或管理接口。

## Consequences

- 广场合同增加刷新 path；收藏 path 按已有 OpenAPI 实现。
- 桌面已登录收藏走 `PUT`/`DELETE /v1/favorites/{id}`；未登录仍只打开登录。
- 管理端可用 `/v1/admin/me` 校验角色；Refresh 仍不进 Web Storage。
- 密码 KDF、Postgres、计费、商店分发仍另开变更。
