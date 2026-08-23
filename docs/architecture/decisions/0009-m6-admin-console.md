# 9. M6 独立管理合同

- Status: accepted
- Date: 2026-08-23
- 关联：[ADR 0008](0008-m5-backend-contract.md) · [管理台](../../specs/admin/spec.md) · [提案](../../changes/m6-admin-console/proposal.md)

## Context

ADR 0008 把管理台排除在 M5 之外。M5 已能提交 `pending` 发布，但没有运营审核。M6 进入标准要求独立管理规格，以及写路径在生产或预发可用。本仓库没有独立生产集群。旧 Spring / `admin-web` 不是现行合同。

## Decision

M6 使用独立 OpenAPI（`admin.yaml`），挂在同一 Axum `backend/` 的 `/v1/admin/*`。管理端是独立 `admin-web/`，不进桌面安装包，不进 Tauri 窗口。本机 `backend/` 是预发，不声称生产。管理员用邮箱 + 密码且须有管理员角色；Refresh 不进 Web Storage。不绑定 QQ / LinuxDo / Google / GitHub。

## Consequences

- 合同原文进 `docs/reference/openapi/admin.yaml` 并登记 INDEX；不把管理路径写进已接受的 `square.yaml`。
- 普通用户 Access 不能改发布状态。审核不改作者本地 SQLite。
- 启动器仍只搜本地库，不请求管理接口。
- OAuth、计费、登录收藏与令牌轮换仍另开变更。
