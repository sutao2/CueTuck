# 3. 第一期纯本地

- Status: accepted
- Date: 2026-08-22
- 关联：[PRD](../../product/prd.md) · [路线图](../../product/roadmap.md)

## Context

旧后端有认证、同步、广场、计费，但与原型的合集、两级分类、匿名下载并不对齐。第一期若同时接旧 API，会被合同拖住。

## Decision

M1–M4 只做本机桌面：SQLite、主窗口、启动器。不接旧 Spring Boot，不实现 OAuth、同步、广场、计费。M5 合同见 [ADR 0008](0008-m5-backend-contract.md)。

## Consequences

- 广场、发布、登录规格先写成目标合同，实现状态为「未开始」。
- 可减少第一期测试面，门禁先覆盖本地与文档。
- 用户第一期不能浏览广场；文档必须写明，不得宣传未做能力。
