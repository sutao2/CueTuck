# 分类

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 实现 |
| 关联 | [数据模型](../../architecture/data-model.md) |

## Purpose

用最多两级分类组织本地内容。本地允许用户新增大分类及小分类；广场使用管理员维护的远端分类字典，与本地分类隔离。发布分类映射见[发布规格](../publish/spec.md)。

## Requirements

### Requirement: 远端字典与本地隔离

广场 MUST 使用公共字典的启用分类及父子关系，不把管理员新建分类写入本地 SQLite。加载失败可保留已取得的字典或旧系统分类，但 MUST 提示配置未更新。选择远端大分类包含其小分类内容；本地空间继续使用本地树。

#### Scenario: 远端新增、下载与发布

- GIVEN 后台新增大分类及小分类，用户本地没有对应分类
- WHEN 用户刷新广场、下载该分类内容并打开发布表单
- THEN 广场可按新分类筛选，发布表单可选择该分类；下载仍保留正文和模型，本地不认识的分类落到未分类
- AND 不覆盖或删除用户本地自定义分类；合集成员同样安全映射，发布历史仍保留远端引用

### Requirement: 远端引用迁移

owner/admin MUST 可显式选择有效同类目标、填写原因并确认版本，迁移公开分类/模型引用后移除源。仅改广场运营元数据及合集成员标签，投稿原始快照和所有正文不变；源保留墓碑及历史重定向，终点再次迁移时压平，不复活源 ID。

#### Scenario: 公开引用与历史保护

- GIVEN 源字典有公开内容、合集成员及历史投稿引用
- WHEN 迁移到有效目标
- THEN 公开标签和版本更新，历史投稿保留原值；以后人工通过时上架到当前目标
- AND 目标有历史重定向时不能直接删除；相关迁移发生在 AI 审核期间则转人工，不能沿用旧分类判定

#### Scenario: 层级和审核策略保护

- GIVEN 源为大分类或当前审核 Skill 仍引用源
- WHEN 迁移
- THEN 大分类只能迁到大分类且子分类重新挂接，同名子分类冲突拒绝；Skill 引用须所有者先显式调整
- AND 源/目标版本冲突或审计失败无部分写入，本地字典不变

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

#### Scenario: 新建入口与校验

- GIVEN 用户处于本地空间的全库、大分类或小分类
- WHEN 点击新建分类
- THEN 弹窗可选择“无（新建大分类）”或已有大分类；全库默认无父级，当前小分类只用于默认选择其父类，不创建第三级
- AND 空白或同一父类下去除首尾空白后的同名分类被拒绝；提交中禁止重复提交和关闭，失败保留输入
- AND 广场没有本地分类新增/删除入口，全部折叠明确命名而不使用减号表示

### Requirement: 删除自定义分类

#### Scenario: 自定义大分类完整使用

- GIVEN 用户在本地全库点击新建分类
- WHEN 选择无父级并输入名称
- THEN 新建非系统大分类，可直接存放内容或新增小分类；同级重名被拒绝
- AND 导出导入及乱序同步仍保留大分类、子分类和内容的关联，不允许第三级

#### Scenario: 删除自定义大分类

- GIVEN 自定义大分类仍有未删除的小分类
- WHEN 尝试删除
- THEN 拒绝并提示先删除小分类，不修改数据
- AND 无子分类的大分类可确认删除，其直接内容移到未分类

#### Scenario: 删除但保留内容

- GIVEN 自定义小分类含提示词及合集
- WHEN 用户点击该分类删除入口并确认
- THEN 该分类软删除，所属提示词与合集移至未分类，正文与合集成员关系保持不变
- AND 若正在查看该分类则转到未分类并刷新数量；取消确认或写入失败不改数据
- AND 系统预置分类不能删除，原生命令同样拒绝

#### Scenario: 删除不复活

- GIVEN 分类已经删除
- WHEN 重启、导出或进行云同步
- THEN 分类列表与导出不包含该分类，同步保留删除墓碑；另一端收到删除也清空该分类的内容归属
- AND 旧库兼容增加分类删除时间，不丢失原有数据

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
| 拒绝第三级 | `desktop/src-tauri` `rejects_grandchild_under_frontend`；`library.test.js` rejects a third-level category；UI 创建同父类的兄弟分类 |
| 本地新增小分类 | `desktop/src-tauri` `creates_user_child_under_office`；`library.test.js` adds a user child category under a parent；`WorkbenchShell.spec.js` adds a local child category under the selected parent |
| 选中大分类 | `desktop/src-tauri` `selecting_parent_lists_child_prompts` |
| 新建入口与校验 | `CategoryActions.spec.js` 全库入口、父分类、同名、IME 与防重复提交；`categoryActions.test.js` 空白与同级同名 |
| 删除但保留内容 | `CategoryActions.spec.js` 确认/取消/失败/数量；`categoryActions.test.js` 内容与合集成员；Rust `category_delete_rolls_back_when_content_update_fails` |
| 删除不复活 | `categoryActions.test.js` 删除墓碑与导出；Rust `category_delete_preserves_content_and_syncs_without_resurrection` |
| 自定义大分类完整使用 | `CategoryActions.spec.js` 大分类与子分类新建、广场隔离；`categoryActions.test.js` 乱序同步与导入；Rust `custom_root_categories_round_trip_and_delete_safely` |
| 删除自定义大分类 | `CategoryActions.spec.js` 子分类删除保护；Rust `custom_root_categories_round_trip_and_delete_safely`；`custom_root_categories_reject_invalid_imports_and_sync_atomically` 校验事务回滚 |
