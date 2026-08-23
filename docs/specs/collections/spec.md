# 合集

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 实现 |
| 关联 | [数据模型](../../architecture/data-model.md) |

## Purpose

把一组相关提示词组织在一起，作为内容区里与单条提示词同级的条目。

## Requirements

### Requirement: 合集是内容不是树节点

系统 MUST NOT 把合集渲染进分类树。合集 MUST 出现在所属小分类的内容区，与提示词混排。

#### Scenario: 筛选小分类

- GIVEN 「人像摄影」下有一条提示词和一个合集
- WHEN 用户选中「人像摄影」
- THEN 内容区同时出现该提示词与该合集
- AND 侧栏树中没有合集行

### Requirement: 创建

系统 MUST 允许在新建流程中选择「单个提示词」或「提示词合集」。合集 MUST 有标题、大小分类，可选封面。

#### Scenario: 新建合集

- GIVEN 用户在本地空间选择新建合集
- WHEN 用户提交名称「人像灵感」和大分类「图片生成」
- THEN 本地存在该合集
- AND 打开合集详情时条目数可以为 0

### Requirement: 成员

系统 MUST 用 `prompts.collection_id` 归属成员。第一期一条提示词最多属于一个合集。

#### Scenario: 向合集添加

- GIVEN 空合集 A 与本地提示词 B
- WHEN 用户把 B 加入 A
- THEN B 的 `collection_id` 为 A
- AND 合集详情列出 B

### Requirement: 封面

系统 MUST 支持无封面、单图封面、多图网格封面。网格封面按资源顺序展示，缺图时用占位，不得因此打不开详情。

#### Scenario: 九宫格缺图

- GIVEN 合集声明网格封面但只有 3 张图
- WHEN 用户打开合集详情
- THEN 详情仍打开
- AND 已有图片可见

## 测试映射

| 场景 | 测试 |
|---|---|
| 筛选小分类 | `desktop/src-tauri` `selecting_parent_lists_child_prompts`（合集走同一分类过滤）；侧栏树无合集行 |
| 新建合集 | `desktop/src-tauri` `creates_empty_collection` |
| 向合集添加 | `desktop/src-tauri` `adds_member_via_collection_id`；`library.test.js` adds a prompt to a collection |
| 九宫格缺图 | `desktop/src-tauri` `persists_grid_cover_refs`；`library.test.js` stores cover refs；`cover.test.js` 缺项填占位；`CollectionDetailModal.spec.js` 3 图 + 6 占位；`WorkbenchShell.spec.js` 卡片预览前 3 张 |
