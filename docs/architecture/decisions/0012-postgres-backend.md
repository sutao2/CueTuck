# 12. 预发后端用本机已有 Postgres，口令用 Argon2id

- Status: accepted
- Date: 2026-08-24
- 关联：[ADR 0008](0008-m5-backend-contract.md) · [ADR 0013](0013-oauth-google-github.md) · [认证](../../specs/auth/spec.md) · [提案](../../changes/postgres-backend/proposal.md)

## Context

M5–M7 的 OpenAPI 跑在内存 HashMap 上，重启即丢。口令是 SHA-256 hex。本机已有 PromptLauncher compose：Postgres `localhost:5432`、Redis `6379`、MinIO `9000`。旧库名 `pl` 是 Spring Flyway，与本仓库 `/v1/` 合同不对齐。人允许表不对时删表重建，并要求完成整个预发后端。

## Decision

在同一 Postgres 实例上使用独立库 `promptark`（用户 `pl`）。本仓库自有 schema，不写入 `pl` 里的 Flyway 表。表缺失或字段不对时允许 `DROP` 后重建。口令校验器为 Argon2id。Refresh 与短时 OAuth 浏览器会话可进 Redis。对象文件进已有 MinIO bucket。单测仍用内存；连上本机库的集成测试必须跑通。不接云同步个人库、Stripe 或商店上架。不是 M10，不声称生产托管。

## Consequences

- `cargo run` 默认连 `postgres://pl:pl@127.0.0.1:5432/promptark`。
- 现行 `/v1/` 路径不变；进程重启后账号、广场、投稿、收藏、设置仍在。
- 启动器与 MCP 仍不请求广场或管理接口。
