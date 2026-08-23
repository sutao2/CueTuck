# 8. M5 改写后端合同

- Status: accepted
- Date: 2026-08-23
- 关联：[ADR 0003](0003-local-first-phase1.md) · [广场](../../specs/square/spec.md) · [认证](../../specs/auth/spec.md) · [发布](../../specs/publish/spec.md) · [提案](../../changes/m5-backend-contract/proposal.md)

## Context

M1–M4 按 ADR 0003 不接旧 Spring Boot。M5 必须在复用旧 API 与改写合同之间单独立项。旧端点与两级分类、合集作内容、匿名下载不对齐。测试门禁要求覆盖率在开工前写死。

## Decision

M5 使用本仓库新 OpenAPI，不对接旧 Spring 路径。账号为邮箱 + 密码；Refresh 只进系统钥匙串。不绑定 QQ / LinuxDo / Google / GitHub。新后端生产代码行覆盖率 ≥ 80%、分支覆盖率 ≥ 70%，API 用 Testcontainers。

## Consequences

- 合同原文进 `docs/reference/openapi/square.yaml` 并登记 INDEX。
- 桌面经 Rust 发 HTTP；Vue 不持有 Refresh。启动器仍只搜本地库。
- 旧 `PromptLauncher` 后端保持只读参考。
- OAuth、管理台、计费另开变更。
