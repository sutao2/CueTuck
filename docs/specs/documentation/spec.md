# 文档体系

| 字段 | 值 |
|---|---|
| 状态 | 现行，M0 已落地 |
| 关联 | [ADR 0004](../../architecture/decisions/0004-documentation-system.md) · [INDEX](../../INDEX.md) |

## Purpose

让规格、计划、代码、测试指向同一套事实，并让 Agent 只通过索引按需读取。

## Requirements

### Requirement: 索引完整

`docs/` 下每个 Markdown 文件 MUST 出现在 `docs/INDEX.md` 的表格中。INDEX 中的每个路径 MUST 指向存在的文件。

#### Scenario: docs-check 捕获孤儿

- GIVEN `docs/orphan.md` 存在但 INDEX 未列出
- WHEN 运行 `./scripts/docs-check`
- THEN 进程失败
- AND 输出包含该路径

### Requirement: 无断链

文档内以 `(` 包裹的相对 `.md` 链接 MUST 指向存在的文件。

#### Scenario: 坏链

- GIVEN 某文档链接到不存在的 `../missing.md`
- WHEN 运行 `./scripts/docs-check`
- THEN 进程失败

### Requirement: ADR 结构

`docs/architecture/decisions/` 下每篇 ADR MUST 含 `Status`、`Context`、`Decision`、`Consequences`。

#### Scenario: 缺段落

- GIVEN 一篇 ADR 没有 Consequences
- WHEN 运行 docs-check
- THEN 进程失败

### Requirement: 先计划后代码

每个应用模块 MUST 在 `docs/plans/` 有实现计划后才能开始写业务代码。M0 MUST NOT 包含应用源代码树。

#### Scenario: 当前仓库

- GIVEN M0 完成
- WHEN 检查仓库根目录
- THEN 存在文档与 `scripts/docs-check`
- AND 不存在 `src/` 应用树

### Requirement: CI 跑同一检查

`main` 的 push 与所有 pull request MUST 运行 `python3 scripts/docs-check`。本地命令与 CI 命令 MUST 一致。

#### Scenario: 工作流存在

- GIVEN 仓库含 `.github/workflows/docs.yml`
- WHEN 打开该工作流
- THEN 其唯一检查步骤调用 `scripts/docs-check`

### Requirement: Agent 常驻规则

仓库 MUST 提供始终生效的 Cursor 规则，要求先读 INDEX、无计划不写应用代码。

#### Scenario: 规则文件

- GIVEN 打开 `.cursor/rules/docs-first.mdc`
- WHEN 查看 frontmatter
- THEN `alwaysApply` 为 true
- AND 正文禁止无计划创建 `src/`

## 测试映射

| 场景 | 测试 |
|---|---|
| docs-check 捕获孤儿 | `scripts/docs-check`（人工/CI） |
| 坏链 | `scripts/docs-check` |
| 缺段落 | `scripts/docs-check` |
| 当前仓库 | 目录约定，M1 前目视 |
| 工作流存在 | `.github/workflows/docs.yml` |
| 规则文件 | `.cursor/rules/docs-first.mdc` |
