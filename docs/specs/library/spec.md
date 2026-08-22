# 本地提示词

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 实现 |
| 关联 | [数据模型](../../architecture/data-model.md) · [变量](../variables/spec.md) |

## Purpose

在本机创建、编辑、删除、搜索和使用提示词，不依赖账号。

## Requirements

### Requirement: 创建

系统 MUST 允许创建单个提示词，写入 SQLite 后立即出现在本地列表。

#### Scenario: 新建并保存

- GIVEN 用户在本地空间
- WHEN 用户创建标题为「测试」的提示词并保存
- THEN 本地库中存在该记录
- AND 列表能搜索到「测试」

### Requirement: 编辑

系统 MUST 在弹窗或等价编辑器中编辑标题、大小分类、模型、正文。正文中的 `{{名称}}` MUST 在下次使用时被识别，无需单独维护变量表。

#### Scenario: 增加变量

- GIVEN 一条不含变量的提示词
- WHEN 用户把正文改为 `你好 {{姓名}}` 并保存
- THEN 使用该提示词时出现「姓名」填写步

### Requirement: 软删除

系统 MUST 使用 `deleted_at` 软删除。默认列表不得出现已删除项。

#### Scenario: 删除后搜索

- GIVEN 列表中有「过期模板」
- WHEN 用户删除它
- THEN 默认搜索不再返回该条
- AND 数据库行仍在且 `deleted_at` 非空

### Requirement: 搜索

系统 MUST 按标题、正文、分类名搜索本地未删除提示词。

#### Scenario: 按正文命中

- GIVEN 正文含「Power Query」
- WHEN 用户在内容区搜索「Power Query」
- THEN 该提示词出现在结果中

### Requirement: 使用计数

系统 MUST 在成功复制或粘贴后增加 `use_count` 并更新 `last_used_at`。

#### Scenario: 复制后计数

- GIVEN 某提示词 `use_count` 为 3
- WHEN 用户从详情或启动器成功复制
- THEN `use_count` 为 4
