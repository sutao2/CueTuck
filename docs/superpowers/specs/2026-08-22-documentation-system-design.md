# 文档体系设计

| 字段 | 值 |
|---|---|
| 状态 | 已接受并落地 |
| 日期 | 2026-08-22 |

本文件记录「为什么这样建文档」，现行规则以 [宪法](../../constitution.md)、[文档规格](../../specs/documentation/spec.md) 和 [ADR 0004](../../architecture/decisions/0004-documentation-system.md) 为准。不要在此复制整份合同。

## 问题

绿场重写需要规格驱动，又要让 Agent 少消耗 token。旧仓库文档与实现脱节（例如仍写 React）。托管文档产品和额外 CLI 会引入第二套真相。

## 决策摘要

组合 Diátaxis（内容职责）、Spec Kit 流程、OpenSpec 的现行/变更分离、Living Docs 索引不变量、MADR。工具只用 git 与 `scripts/docs-check`。

## 永久 vs 临时

永久：宪法、PRD、路线图、术语、现行规格、架构、ADR、门禁、INDEX、操作指南。  
临时：changes、调研、已完成计划。  
不落盘：聊天稿、重复进度百分比、同一事实的第二份拷贝。

## 范围外

本设计不包含应用模块的实现计划。那是 M0 关闭后 `docs/plans/` 的工作。
