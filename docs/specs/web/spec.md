# 浏览器工作台

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M9 目标 |
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

## 测试映射

| 场景 | 测试 |
|---|---|
| 桌面包不含 Web 工作台 | `desktop/src/platform/packageIsolation.test.js` does not depend on or bundle web workbench |
| 窄桌面可收起侧栏 | `web/src/WebApp.spec.js` keeps local space when the sidebar is collapsed |
| 不声称已同步 | `web/src/WebApp.spec.js` does not claim the browser library is synced to desktop sqlite |
| 新建出现在列表 | `web/src/WebApp.spec.js` creates a memory prompt and lists it without claiming desktop sync |
| 点开看到正文 | `web/src/WebApp.spec.js` opens a memory prompt and shows its body |
