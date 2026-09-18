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

合集搜索多选与本地批量整理 MUST 遵循[工作台优化计划](../../plans/2026-09-12-workbench-usability.md)切片 3；每条仍只属于一个合集，加入前明确移动归属。

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

### Requirement: 可维护的合集

合集详情 MUST 能打开或使用成员、从全库选择成员、移除成员、编辑合集标题/分类/封面并软删除合集。移除成员或删除合集 MUST NOT 删除提示词正文；删除合集时成员解除归属。移动成员后原合集与目标合集的计数 MUST 同时更新。不存在或已删除的合集 MUST NOT 接收成员。

#### Scenario: 跨分类管理成员

- GIVEN 合集在图片分类，待加入的提示词在软件分类
- WHEN 打开合集添加该提示词，再将其移除
- THEN 不受当前搜索和分类筛选限制，成员能被加入并打开使用
- AND 移除后提示词仍在本地库

#### Scenario: 编辑与删除合集

- GIVEN 合集有一条成员
- WHEN 编辑标题和封面后再删除合集
- THEN 编辑保存可再次读取，删除后默认列表不再显示合集
- AND 成员仍存在且 collection_id 为空

### Requirement: 读取与成员操作反馈

详情 MUST 区分读取中、读取失败与真实空合集。读取未成功时不可编辑或管理成员，但可以返回；旧请求 MUST NOT 覆盖返回后重新打开的页面（包括相同合集）。读取失败 MUST 提供只读重试。

成员写入期间 MUST 防止重复提交与离开；写入失败 MUST 保留选择。写入成功后刷新失败 MUST 明确告知已加入或已移出，重试 MUST 仅重新读取，不重复写入。

#### Scenario: 返回后重新打开

- GIVEN 合集读取尚未完成
- WHEN 返回后打开同一个或另一个合集，旧请求才结束
- THEN 旧成功或失败均不改变当前页面；当前请求独立显示加载、结果或重试

#### Scenario: 成员操作部分成功

- GIVEN 加入或移出已写入，随后读取失败
- WHEN 用户查看反馈并重试
- THEN 显示操作已完成和刷新失败，只重新读取
- AND 写入失败时保留原选择，忙碌期间重复提交不增加写入次数

## 测试映射

| 场景 | 测试 |
|---|---|
| 返回后重新打开 / 成员操作部分成功 | `CollectionFeedback.spec.js` 8 项延迟/故障注入；`CollectionDetailModal.spec.js` 确认成员更新后恢复焦点 |
| 筛选小分类 | `desktop/src-tauri` `selecting_parent_lists_child_prompts`（合集走同一分类过滤）；侧栏树无合集行 |
| 新建合集 | `desktop/src-tauri` `creates_empty_collection` |
| 向合集添加 | `desktop/src-tauri` `adds_member_via_collection_id`；`library.test.js` adds a prompt to a collection |
| 九宫格缺图 | `desktop/src-tauri` `persists_grid_cover_refs`；`library.test.js` stores cover refs；`cover.test.js` 缺项填占位；`CollectionDetailModal.spec.js` 3 图 + 6 占位；`WorkbenchShell.spec.js` 卡片预览前 3 张 |

## 合集内创作与封面布局（2026-09-18）

- 在合集详情可新建正文和附件，原生事务一次保存，`source=collection` 标识合集专属成员；独立提示词列表、候选列表不展示，已有提示词加入不改变来源。
- GIVEN 合集专属成员；WHEN 移出或删除合集；THEN 保留正文及附件，来源转为 local，作为独立提示词出现；编辑器和成员选择区提前说明。
- GIVEN 创建失败；WHEN 重试；THEN 不遗留半创建的独立提示词。
- 封面由统一组件展示；九宫格固定三行三列、整体正方形，详情最大宽 480px，卡片正常文档流，列表显示缩略图，单图保留比例。缺失格子使用占位。
