# 1. 在旁边新建独立仓库

- Status: accepted
- Date: 2026-08-22
- 关联：[宪法](../../constitution.md)

## Context

旧 PromptLauncher 已有可运行的 Vue/Tauri/Spring Boot 产品，但主窗口要按「提示词软件 2」完全重写。在旧仓库里改会造成现行实现、过时 ADR 和新 IA 缠在一起。

## Decision

在 `IdeaProjects/PromptArk` 新建 git 仓库。旧 `PromptLauncher` 保留只读参考，不重写其历史。

## Consequences

- 启动器与原型资产从旧仓库复制，不在旧仓库继续改主应用。
- 两套仓库并存，Agent 必须以本仓库文档为现行合同。
- 旧仓库的发行 RC 工作与本重写解耦。
