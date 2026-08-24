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

## 测试映射

| 场景 | 测试 |
|---|---|
| 桌面包不含 Web 工作台 | 未开始 |
| 窄桌面可收起侧栏 | 未开始 |
| 不声称已同步 | 未开始 |
