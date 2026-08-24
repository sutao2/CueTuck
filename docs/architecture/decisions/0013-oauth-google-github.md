# 13. 预发后端提供 Google 与 GitHub 登录

- Status: accepted
- Date: 2026-08-24
- 关联：[ADR 0008](0008-m5-backend-contract.md) · [ADR 0012](0012-postgres-backend.md) · [认证](../../specs/auth/spec.md)

## Context

ADR 0008 规定邮箱密码，并写明不绑定 QQ / LinuxDo / Google / GitHub。人已选定 Google 与 GitHub，并要求复用旧 PromptLauncher 的授权码流程与环境变量（`PL_GOOGLE_*` / `PL_GITHUB_*`）。QQ 与 LinuxDo 仍未选定。路径仍走本仓库 `/v1/`，不接旧 `/api/v1/auth`。

## Decision

本仓库 `backend/` 增加 Google 与 GitHub 授权码登录。邮箱密码仍可用。不接 QQ / LinuxDo。不把旧 Spring 路径当作现行合同。桌面设置页未接线前可以不展示绑定入口，但 API 必须真实可用，不得发假请求。

## Consequences

- OpenAPI 增加 `/v1/session/oauth/*`。
- 口令可为空的 OAuth 账号不得用假密码登录。
- ADR 0008 的 OpenAPI 改写与邮箱密码仍有效；其中「不绑定 Google / GitHub」由本篇取代。
