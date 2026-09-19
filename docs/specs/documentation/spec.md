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

### Requirement: API 合同与真实路由一致

`node scripts/api-contract-check.mjs` MUST 双向核对后端显式注册的路径/HTTP 方法与 `square.yaml`、`admin.yaml`，在集中 frontend 验证与 full-regression CI 中执行。路径参数名称可不同；未知路由注册形式 MUST 报错，不能默默跳过。请求/响应字段、认证和业务行为仍由处理器审阅及对应测试验证。

#### Scenario: 新增或残留操作

- GIVEN 后端新增未记入 OpenAPI 的路径或方法，或文档保留已移除的操作
- WHEN 执行集中 frontend 验证
- THEN API 合同检查失败并列出缺少的一侧及操作
- AND 索引/断链检查通过不能抵消该失败

`docs-check` 只检查索引、Markdown 相对链接和 ADR 结构，不证明内容与实现一致，也不验证 HTTP 合同。测试门禁的实际接入情况与未落实要求见[测试门禁](../../reference/test-gates.md)。

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

### Requirement: 仓库入口阶段诚实

仓库根 `README.md` MUST 标明 M9 已关闭。MUST NOT 把已接通的个人库同步、OAuth、自动更新写成尚未接通。MUST 仍标明无商店包。`CLAUDE.md` MUST NOT 把已接通的同步、更新、账单列为未接通示例。

#### Scenario: 入口不把已接通写成未接通

- GIVEN 打开仓库根 README 与 CLAUDE
- WHEN 阅读阶段与能力说明
- THEN README 标明 M9 已关闭
- AND 不把云同步、OAuth、自动更新安装写成不得假装接通
- AND 仍标明无商店包
- AND CLAUDE 不把同步、更新、账单列为未接通示例

## 测试映射

| 场景 | 测试 |
|---|---|
| docs-check 捕获孤儿 | `scripts/docs-check`（人工/CI） |
| 坏链 | `scripts/docs-check` |
| 缺段落 | `scripts/docs-check` |
| 新增或残留操作 | `scripts/api-contract-check.test.mjs` 未记载路径/方法、错误方法、重复操作与未知注册形式；`scripts/api-contract-check.mjs` 实际仓库对照 |
| 当前仓库 | 目录约定，M1 前目视 |
| 工作流存在 | `.github/workflows/docs.yml` |
| 规则文件 | `.cursor/rules/docs-first.mdc` |
| 入口不把已接通写成未接通 | `packageIsolation.test.js` does not claim the repo entry still stops at M8 or unconnected sync |
