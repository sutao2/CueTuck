# 14. 不再按第一期冻结剩余产品

- Status: accepted
- Date: 2026-08-25
- 关联：[宪法](../../constitution.md) · [PRD](../../product/prd.md) · [ADR 0003](0003-local-first-phase1.md) · [ADR 0012](0012-postgres-backend.md)

## Context

M0–M9 与预发后端、客户端 OAuth 已关闭。PRD 与宪法仍按「第一期 / 后期」把个人库云同步、自动更新安装、账单写成另立项。2026-08-25 人要求继续开发，不分第一期第二期，计划完成整个项目。本地优先、不假装、不改启动器为覆盖层，这些原则仍有效。

## Decision

剩余工作按完整产品排队实现，不再用「第一期不做」挡住已写在设置与原型里的能力。仍是切片队列，不是新的 M10。[ADR 0003](0003-local-first-phase1.md) 只描述 M1–M4 当时的范围，不再约束此后切片。

仍不做：原生移动端、QQ / LinuxDo、ChatGPT 插件、把启动器改成覆盖层、在旧仓库改主应用。商店上架与生产托管在没有账号与发行证据前不得声称已完成。[ADR 0012](0012-postgres-backend.md) 的独立库 `promptark` 与 Argon2id 仍有效；其中「不接云同步 / Stripe」由本篇取代。

## Consequences

- 宪法「第一期边界」改为产品边界。
- 个人库云同步、自动更新安装、预发账单进入 `docs/plans/` 队列。
- Windows NSIS 验证仍受 GitHub 额度限制，通过前不得勾选 Windows QA。
