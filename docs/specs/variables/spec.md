# 变量与使用向导

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 主窗口 / M3 启动器 |
| 关联 | [本地提示词](../library/spec.md) · [启动器](../launcher/spec.md) |

## Purpose

从正文解析 `{{变量名}}`，在使用前逐步填写并生成最终文本。不单独维护变量列表。

## Requirements

### Requirement: 解析规则

系统 MUST 把 `{{` 与 `}}` 之间的 trim 后文本当作变量名。同名变量 MUST 只填写一次。空名称 MUST 忽略。

#### Scenario: 重复变量

- GIVEN 正文为 `为 {{产品}} 写介绍，再次强调 {{产品}}`
- WHEN 进入使用向导
- THEN 只有一个「产品」填写步
- AND 两处都替换为同一值

#### Scenario: 无变量

- GIVEN 正文不含 `{{`
- WHEN 用户使用该提示词
- THEN 跳过填写，直接进入预览或复制

### Requirement: 主窗口向导

主窗口使用流程 MUST 按：可选语言步（若存在中英两份正文）→ 逐个变量 → 预览 → 复制。每次只强调一个变量。Enter 进入下一步，Shift+Enter 换行。

#### Scenario: 逐步填写

- GIVEN 正文含 `{{城市}}` 与 `{{天数}}`
- WHEN 用户开始使用
- THEN 先只要求填写「城市」
- AND 下一步才是「天数」
- AND 预览展示替换后的完整正文

### Requirement: 启动器填写

启动器填写态可以一屏展示全部变量，但解析规则 MUST 与主窗口相同。渲染函数 MUST 共用同一实现。

#### Scenario: 同源渲染

- GIVEN 同一提示词与同一组变量值
- WHEN 分别在主窗口向导与启动器生成最终文本
- THEN 两段文本完全一致

### Requirement: 未填变量

未填写的变量 MUST 在最终文本中保留 `{{名称}}` 或替换为空字符串。选定一种行为后不得混用。本仓库选定：**未填保留 `{{名称}}`**，以便用户发现漏填。

#### Scenario: 漏填

- GIVEN 变量「受众」未填
- WHEN 用户在预览后复制
- THEN 最终文本仍包含 `{{受众}}`

### Requirement: 受设置控制的双语与建议

设置「提示词双语版本」关闭时，向导 MUST 跳过语言步，但仍 MUST 保留库里已有的中英正文，不得删除。变量智能建议关闭时 MUST 不提供建议；打开时 MUST NOT 把提示词正文传到本机以外。

#### Scenario: 关闭双语不删正文

- GIVEN 一条提示词同时有中文与英文正文，且双语开关为关
- WHEN 用户开始使用
- THEN 不出现语言选择步
- AND 两条正文都还在库里

## 测试映射

| 场景 | 测试 |
|---|---|
| 重复变量 | `desktop/src/lib/renderPrompt.test.js` dedupes repeated variables |
| 无变量 | `desktop/src/components/UsePromptModal.spec.js` skips fill and previews when the prompt has no variables |
| 逐步填写 | `desktop/src/components/UsePromptModal.spec.js` asks for one variable at a time then previews the filled text |
| 同源渲染 | 启动器与工作台共用 `desktop/src/lib/renderPrompt.js`；`LauncherApp.spec.js` 填写态预览保留 `{{姓名}}` |
| 漏填 | `desktop/src/lib/renderPrompt.test.js` keeps unfilled placeholders |
| 关闭双语不删正文 | `WorkbenchShell.spec.js` keeps prompt content when bilingual is turned off |
