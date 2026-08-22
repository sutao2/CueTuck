# 4. 文档体系

- Status: accepted
- Date: 2026-08-22
- 关联：[INDEX](../../INDEX.md) · [文档规格](../../specs/documentation/spec.md)

## Context

需要规格驱动开发，并让 Agent 少读文件。单独采用 Spec Kit 或 OpenSpec CLI 会增加工具漂移。

## Decision

用仓库内 Markdown 组合：Diátaxis 分类内容、Spec Kit 流程（宪法 → 规格 → 计划 → 实现 → converge）、OpenSpec 的现行规格/变更分离、Living Docs 的索引不变量、MADR 决策记录。不安装这些 CLI。`docs/INDEX.md` 是导航入口；`scripts/docs-check` 检查孤儿、断链和 ADR 必备段落。

## Consequences

- 应用代码不得早于对应计划和规格。
- 进度只写在路线图，不写独立进度百分比文。
- 文档门禁是每轮提交的硬条件。
