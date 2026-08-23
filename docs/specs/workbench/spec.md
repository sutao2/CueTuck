# 工作台

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M1–M2 实现 |
| 来源 | 原型 `index.html` 主壳 |
| 关联 | [说明：设计源](../../explanation/ui-source.md) |

## Purpose

主窗口的桌面工作台：顶栏、侧栏、内容区、底栏。第一期只启用本地空间。

## Requirements

### Requirement: 壳层结构

系统 MUST 提供顶栏、可拖宽侧栏、内容区、22px 级底栏，布局对齐原型，而不是旧 AppShell 导航。

#### Scenario: 打开应用

- GIVEN 用户启动桌面应用
- WHEN 主窗口显示
- THEN 可见顶栏品牌与当前位置、侧栏、内容区与底栏

### Requirement: 空间

系统 MUST 在侧栏顶部提供「提示词广场」与「本地提示词」两个空间入口。第一期选择广场时 MUST 说明在线能力未开放，并保持本地可用。

#### Scenario: 第一期点击广场

- GIVEN 当前为第一期构建
- WHEN 用户点击「提示词广场」
- THEN 不加载远端列表
- AND 用户看到明确的未开放说明
- AND 可一键回到本地

### Requirement: 侧栏分类

系统 MUST 以「大分类 → 小分类」两级树展示分类，合集不得出现在树上。

#### Scenario: 展开大分类

- GIVEN 「软件开发」下有小分类
- WHEN 用户展开「软件开发」
- THEN 其小分类可见
- AND 合集只出现在内容区

### Requirement: 内容头与视图

系统 MUST 提供搜索、排序或筛选、网格/列表切换。本地主操作是新建，不是发布。

#### Scenario: 切换网格列表

- GIVEN 内容区有至少一条提示词
- WHEN 用户切换到列表视图
- THEN 同一批结果以行展示而不是卡片网格

### Requirement: 宿主窗口样式

系统 MUST 按宿主操作系统画窗口控件与快捷键记号。原型只定四区节奏与内容，不定 Windows 风窗框。macOS 上 MUST 使用系统红绿灯（左上），自定义顶栏 MUST 为红绿灯留出 inset，不得把无框矩形窗 + 右侧工具簇当成成品。快捷键展示 MUST 用 Mac 符号（如 `⌃Space`），不得写 `Ctrl Space`。

#### Scenario: macOS 主窗口

- GIVEN 用户在 macOS 打开桌面主窗口
- WHEN 窗口显示
- THEN 左上为系统红绿灯，可拖区域不与按钮重叠
- AND 顶栏快捷键记号为 `⌃Space` 而不是 `Ctrl Space`
- AND 不得出现 Windows 风格的右侧最小化 / 最大化 / 关闭

### Requirement: 底栏

系统 MUST 显示本地库状态与本地条数。第一期不显示同步队列为「连接广场」。

#### Scenario: SQLite 就绪

- GIVEN 本地库初始化成功
- WHEN 工作台渲染底栏
- THEN 显示库已就绪与当前未删除的本地提示词数量

## 测试映射

| 场景 | 测试 |
|---|---|
| 打开应用 | `desktop/src/components/WorkbenchShell.spec.js` renders four chrome regions；renders prototype sidebar chrome |
| 第一期点击广场 | `desktop/src/components/WorkbenchShell.spec.js` keeps square offline on first phase |
| SQLite 就绪 | `desktop/src-tauri` `status_is_ready_after_initialize` |
| 展开大分类 | `desktop/src/components/WorkbenchShell.spec.js` loads preset categories into the tree；`lists_children_under_software` |
| 切换网格列表 | `WorkbenchShell.spec.js` shows the same prompts as rows in list view |
| macOS 主窗口 | `WorkbenchShell.spec.js` uses mac chrome on macos；`windowChrome.test.js` gives traffic-light inset and glyph shortcut on macos |
