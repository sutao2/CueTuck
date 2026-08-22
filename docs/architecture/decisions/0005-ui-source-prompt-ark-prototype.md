# 5. 主窗口以提示词软件 2 为设计源

- Status: accepted
- Date: 2026-08-22
- 关联：[说明：设计源](../../explanation/ui-source.md)

## Context

旧主窗口是 Atelier Zero 黑白画廊风，路由分库/广场/设置。新原型是系统原生工作台：顶栏、广场/本地、两级分类、卡片、底栏、弹窗编辑。

## Decision

主窗口布局、组件节奏和文案以 `../PromptLauncher/提示词软件 2` 为准。设计 token 从该目录的 `styles.css` 迁移，不把旧 `app.css` 当主窗口主题。

## Consequences

- 旧 86 屏 HTML 原型不再是主窗口事实源。
- 合集、两级分类、逐步变量向导进入产品合同。
- 启动器不受本 ADR 约束，见 [ADR 0002](0002-preserve-current-launcher.md)。
