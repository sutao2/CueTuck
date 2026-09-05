# 分类

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 实现 |
| 关联 | [数据模型](../../architecture/data-model.md) |

## Purpose

用固定两级分类组织本地内容。广场发布时必须映射到系统大分类（M5）。本地允许用户新增小分类。

## Requirements

### Requirement: 两级上限

系统 MUST 只允许大分类与小分类两级。小分类 MUST NOT 再有子分类。

#### Scenario: 拒绝第三级

- GIVEN 已存在小分类「前端工程」
- WHEN 用户尝试在其下再创建子分类
- THEN 操作失败
- AND 树仍为两级

### Requirement: 预置大分类

新库初始化 MUST 写入十个系统大分类：软件开发、图片生成、视频创作、办公效率、内容写作、产品设计、市场营销、数据分析、教育学习、生活助手。

#### Scenario: 空库首次打开

- GIVEN 全新本地库
- WHEN 工作台加载分类树
- THEN 上述十大分类均存在且 `is_system` 为真

### Requirement: 预置小分类

系统 MUST 为每个大分类提供原型中的首包小分类（如软件开发下的网站开发、前端工程、后端与数据库、测试与审查）。用户 MUST 能在本地大分类下新增小分类。

#### Scenario: 本地新增小分类

- GIVEN 大分类「办公效率」
- WHEN 用户新增小分类「周报」
- THEN 树中「办公效率」下出现「周报」
- AND `is_system` 为假

### Requirement: 筛选

选中大分类 MUST 列出其下全部小分类的提示词与合集。选中小分类 MUST 只列出该小分类。

#### Scenario: 选中大分类

- GIVEN 「图片生成」下「人像摄影」与「商品视觉」各有一条
- WHEN 用户选中「图片生成」
- THEN 两条都出现
- AND 其他大分类内容不出现

## 测试映射

### Requirement: 分类上下文与数量

分类树 MUST 显示本地未删除提示词与合集的条目数量，大分类含子分类。未分类 MUST 有可选择入口；新建 MUST 继承当前分类，编辑 MUST 保留原分类。广场分类数量未取得全量统计时不得显示本地数量。

#### Scenario: 分类下新建

- GIVEN 用户选中「人像摄影」
- WHEN 打开新建并保存标题正文
- THEN 新记录属于「人像摄影」并立即出现在当前列表

#### Scenario: 数量与未分类

- GIVEN 图片分类下一条提示词、一个合集，另有一条未分类提示词
- WHEN 查看分类树
- THEN 图片分类计数为 2，未分类计数为 1
- AND 搜索或切换排序不改变分类总数

#### Scenario: 编辑大分类记录

- GIVEN 一条提示词直接属于「图片生成」
- WHEN 打开编辑并保存
- THEN 下拉正确显示大分类且所属分类不丢失

回归：`WorkbenchShell.spec.js` 分类上下文与数量；`library.test.js` 未分类过滤。

| 场景 | 测试 |
|---|---|
| 空库首次打开 | `desktop/src-tauri` `seeds_ten_system_categories` |
| 拒绝第三级 | `desktop/src-tauri` `rejects_grandchild_under_frontend`；`library.test.js` rejects a third-level category；`WorkbenchShell.spec.js` refuses a third-level category from a child |
| 本地新增小分类 | `desktop/src-tauri` `creates_user_child_under_office`；`library.test.js` adds a user child category under a parent；`WorkbenchShell.spec.js` adds a local child category under the selected parent |
| 选中大分类 | `desktop/src-tauri` `selecting_parent_lists_child_prompts` |
