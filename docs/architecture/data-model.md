# 数据模型

| 字段 | 值 |
|---|---|
| 状态 | 现行（目标模型） |
| 关联 | [架构](overview.md) · [分类规格](../specs/categories/spec.md) · [合集规格](../specs/collections/spec.md) |

第一期本地库由 Rust `rusqlite` 打开 `app_data_dir/promptark.sqlite`。初始化会建 `settings`、`categories`、`collections`、完整 `prompts`，并幂等写入十大系统分类及首包小分类。见 [ADR 0007](decisions/0007-sqlite-access.md)。

## 本地表

### prompts

| 列 | 说明 |
|---|---|
| id | UUID |
| title | 标题 |
| summary | 列表摘要 |
| content | 正文，变量以 `{{名称}}` 写在正文里 |
| category_id | 小分类，可空表示未分类 |
| collection_id | 所属合集，可空 |
| model | 适用模型标签，本地可空 |
| source | `local` / `downloaded` |
| remote_id | 后期用，第一期空 |
| version | 本地版本号 |
| use_count | 使用次数 |
| last_used_at | 最近使用 |
| created_at / updated_at | 时间 |
| deleted_at | 软删除 |

不单独建 `prompt_variables` 表。变量以正文解析为准，见 [变量规格](../specs/variables/spec.md)。

### categories

| 列 | 说明 |
|---|---|
| id | UUID |
| parent_id | 空表示大分类 |
| name | 名称 |
| icon | 可选 |
| is_system | 系统预置分类 |
| sort_order | 排序 |

只允许两级：大分类 → 小分类。小分类不得再有子节点。

### collections

| 列 | 说明 |
|---|---|
| id | UUID |
| title | 名称 |
| description | 简介 |
| category_id | 归属小分类 |
| cover_type | `none` / `single` / `grid` |
| cover_json | 封面资源引用 |
| created_at / updated_at | 时间 |
| deleted_at | 软删除 |

合集内提示词用 `prompts.collection_id` 关联。第一期不做合集与提示词多对多。

### tags / prompt_tags

可选。第一期列表筛选以分类和全文搜索为主；标签表可在 M2 计划里决定是否落地。

### assets

| 列 | 说明 |
|---|---|
| id | UUID |
| owner_type | `prompt` / `collection` |
| owner_id | 所属 id |
| local_path | 本机路径 |
| mime_type | 类型 |

### settings

| 列 | 说明 |
|---|---|
| key | 设置键 |
| value_json | JSON |

键名与含义只写在 [设置规格](../specs/settings/spec.md)，本表不重复。M8 增加键不得删已有 `theme` 与启动器快捷键。

### sync_jobs

第一期不建。M5 再引入。

## 预置大分类

软件开发、图片生成、视频创作、办公效率、内容写作、产品设计、市场营销、数据分析、教育学习、生活助手。小分类首包见 [分类规格](../specs/categories/spec.md)。

## 标识

本地主键使用 UUID。删除一律软删除。物理清理策略在设置规格中定义。
