# 浏览器工作台

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M9 实现 |
| 关联 | [工作台](../workbench/spec.md) · [ADR 0011](../../architecture/decisions/0011-web-and-mcp.md) |

## Purpose

在桌面浏览器里使用提示方舟工作台。布局随窗口宽度伸缩，像 Notion 那样适配常见桌面宽度。不是原生移动应用。

## Requirements

### Requirement: 独立于桌面安装包

浏览器工作台 MUST 是独立 Web 应用。桌面安装包 MUST NOT 包含 `web/` 源码。

#### Scenario: 桌面包不含 Web 工作台

- GIVEN 构建桌面客户端
- WHEN 检查安装包与 `desktop/` 依赖
- THEN 产物不含独立 `web/` 应用源码

### Requirement: 流体桌面布局

在常见桌面浏览器宽度下，工作台 MUST 保持顶栏、侧栏、内容区可用。侧栏 MUST 可收起以便窄桌面窗口。MUST NOT 把原生手机应用当作交付物。

#### Scenario: 窄桌面可收起侧栏

- GIVEN 浏览器窗口窄于工作台默认宽屏
- WHEN 用户打开浏览器工作台
- THEN 仍能进入本地空间
- AND 侧栏可以收起或叠放，不挡住主操作

### Requirement: 不假装本机库已同步

浏览器工作台 MUST NOT 声称正在读写桌面那份 SQLite。云同步未接通前，浏览器库与桌面库可以不同。

#### Scenario: 不声称已同步

- GIVEN 云同步尚未提供
- WHEN 用户在浏览器工作台查看数据说明
- THEN 不出现「已与桌面库同步」或等价假状态

### Requirement: 标签页内存库

浏览器工作台 MUST 能在当前标签页新建提示词，并立即出现在本地列表。MUST 把数据放在内存里。MUST NOT 写入桌面 SQLite。刷新后丢失是预期，不得写成已持久化到桌面。

#### Scenario: 新建出现在列表

- GIVEN 浏览器工作台本地空间为空
- WHEN 用户创建标题为「测试」的提示词并保存
- THEN 列表出现「测试」
- AND 数据说明仍写尚未与桌面 SQLite 同步

### Requirement: 打开查看正文

浏览器工作台 MUST 允许从本地列表打开一条提示词并看到正文。MUST NOT 为此去读桌面 SQLite。

#### Scenario: 点开看到正文

- GIVEN 列表中有标题「测试」、正文「你好」的提示词
- WHEN 用户点开该条
- THEN 内容区显示正文「你好」

### Requirement: 编辑内存提示词

浏览器工作台 MUST 能修改当前标签页里已有提示词的标题与正文。保存后列表 MUST 立即显示新标题。MUST NOT 写入桌面 SQLite。

#### Scenario: 编辑后列表更新

- GIVEN 列表中有标题为「测试」的提示词
- WHEN 用户打开该条，把标题改为「已改」并保存
- THEN 列表出现「已改」
- AND 列表不再以「测试」作为该条标题
- AND 数据说明仍写尚未与桌面 SQLite 同步

### Requirement: 浏览器使用向导

浏览器工作台 MUST 按与桌面相同的规则解析 `{{名称}}`：同名只填一次，未填保留 `{{名称}}`。有变量时 MUST 一次只问一个；无变量时 MUST 跳过填写进入预览。MUST NOT 把正文传到本机以外。

#### Scenario: 逐步填写

- GIVEN 正文含 `{{城市}}` 与 `{{天数}}`
- WHEN 用户开始使用
- THEN 先只要求填写「城市」
- AND 下一步才是「天数」
- AND 预览展示替换后的完整正文

#### Scenario: 无变量直接预览

- GIVEN 正文不含 `{{`
- WHEN 用户开始使用
- THEN 跳过填写，直接进入预览

#### Scenario: 漏填保留占位

- GIVEN 变量「受众」未填
- WHEN 用户进入预览
- THEN 最终文本仍包含 `{{受众}}`

### Requirement: 浏览器预发广场

浏览器工作台 MUST 能请求本仓库预发广场。失败时 MUST 给出非阻断离线说明并可回到本地。匿名下载 MUST 写入当前标签页内存库，MUST NOT 声称写入桌面 SQLite。未登录收藏 MUST NOT 因此新增内存副本。

#### Scenario: 浏览器广场离线

- GIVEN 预发 API 不可用
- WHEN 用户打开浏览器广场
- THEN 显示离线说明
- AND 可以回到本地空间

#### Scenario: 浏览器匿名下载

- GIVEN 预发广场有一条「自然光群像」
- WHEN 用户匿名下载
- THEN 当前标签页内存库出现该标题
- AND 说明仍写尚未与桌面 SQLite 同步
- AND 不出现「已写入本机 SQLite」

## 测试映射

| 场景 | 测试 |
|---|---|
| 桌面包不含 Web 工作台 | `desktop/src/platform/packageIsolation.test.js` does not depend on or bundle web workbench |
| 窄桌面可收起侧栏 | `web/src/WebApp.spec.js` keeps local space when the sidebar is collapsed |
| 不声称已同步 | `web/src/WebApp.spec.js` does not claim the browser library is synced to desktop sqlite |
| 新建出现在列表 | `web/src/WebApp.spec.js` creates a memory prompt and lists it without claiming desktop sync |
| 点开看到正文 | `web/src/WebApp.spec.js` opens a memory prompt and shows its body |
| 编辑后列表更新 | `web/src/WebApp.spec.js` updates a memory prompt title in the list after edit |
| 逐步填写 | `web/src/WebApp.spec.js` fills wizard variables one at a time then previews and copies |
| 无变量直接预览 | `web/src/WebApp.spec.js` skips fill and previews when there are no variables |
| 漏填保留占位 | `web/src/renderPrompt.spec.js` keeps unfilled placeholders |
| 浏览器广场离线 | `web/src/WebApp.spec.js` shows a square offline notice and can return to local |
| 浏览器匿名下载 | `web/src/WebApp.spec.js` downloads a square prompt into the memory library without claiming sqlite |
