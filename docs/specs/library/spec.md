# 本地提示词

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 实现 |
| 关联 | [数据模型](../../architecture/data-model.md) · [变量](../variables/spec.md) |

## Purpose

在本机创建、编辑、删除、搜索和使用提示词，不依赖账号。

## Requirements

### Requirement: 本地参考资料

提示词 MUST 支持独立图片/文档附件，不把文件混入正文或复制结果。支持格式与限额见[实施计划](../../plans/2026-09-08-client-assets.md)。保存 MUST 在同一 SQLite 事务写入正文与文件；取消不保存草稿。图片和纯文本在页面预览，其余文档导出后由用户自行打开，不执行文件或渲染主动内容。读取失败不得用空附件覆盖原文件。

#### Scenario: 文件保存与离开

- GIVEN 提示词有图片及 PDF 附件
- WHEN 编辑、保存并重新打开，或修改附件后返回
- THEN 已保存文件可查看/导出，未保存变化需确认放弃；失败保留原内容
- AND 文件删除仅在保存后生效，软删除提示词仍保留文件于本地备份

#### Scenario: 备份与私人文件边界

- GIVEN 提示词附有私人文件
- WHEN JSON 导入导出、SQLite/ZIP 备份或云同步/发布
- THEN 本地备份保留文件；普通同步和投稿不携带附件；显式手动私有附件同步见[同步规格](../sync/spec.md)
- AND 远端变更不得删除本机附件，显式同步只补齐缺少文件

### Requirement: 编辑防误关闭

主窗口保存/复制结果 MUST 遵循 [P3a 计划](../../plans/2026-09-08-client-operation-feedback.md)：写入成功与后续刷新失败分开反馈，刷新重试不重复写入或复制；编辑页保存快捷键与按钮共享校验和忙碌保护。该片不改变独立启动器生命周期。

#### Scenario: 未保存编辑离开

- GIVEN 提示词或合集编辑内容与打开时不同
- WHEN 点击返回、取消、切换侧栏目标或按 Escape
- THEN 先询问是否放弃，继续编辑保留全部输入；未改动可直接关闭
- AND 保存进行中禁止关闭与删除，保存失败保留输入；成功保存按原流程退出

### Requirement: 创建

系统 MUST 允许创建单个提示词，写入 SQLite 后立即出现在本地列表。

#### Scenario: 新建并保存

- GIVEN 用户在本地空间
- WHEN 用户创建标题为「测试」的提示词并保存
- THEN 本地库中存在该记录
- AND 列表能搜索到「测试」

### Requirement: 编辑

系统 MUST 在主窗口右侧编辑页面中编辑标题、大小分类、模型、正文，不采用模态弹窗。返回保留原查询、筛选和滚动。正文中的 `{{名称}}` MUST 在下次使用时被识别，无需单独维护变量表。

#### Scenario: 增加变量

- GIVEN 一条不含变量的提示词
- WHEN 用户把正文改为 `你好 {{姓名}}` 并保存
- THEN 使用该提示词时出现「姓名」填写步

### Requirement: 软删除

系统 MUST 使用 `deleted_at` 软删除。默认列表不得出现已删除项。

删除交互（提示词、合集及分类结果反馈）MUST 遵循[删除反馈切片](../../plans/2026-09-08-local-completion.md)的 P0 场景：提示词/合集先用非模态确认条明确对象，确认才执行；成功/失败固定提示可见，失败可重试，刷新失败不误报为写入失败。

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

#### Scenario: 复制失败不计数

- GIVEN 剪贴板拒绝写入
- WHEN 用户确认复制
- THEN 显示可重试的失败提示并保留填写内容
- AND 不增加使用次数，不离开使用页面

## 测试映射

| 场景 | 测试 |
|---|---|
| 未保存编辑离开 | `ClientPolish.spec.js` 关闭/遮罩/Escape、继续编辑、放弃、未改动及保存中禁止关闭 |
| 新建并保存 | `desktop/src-tauri` `creates_prompt_and_lists_it`；`desktop/src/platform/library.test.js` |
| 增加变量 | `desktop/src/lib/renderPrompt.test.js` dedupes repeated variables |
| 编辑模型 | `library.test.js` persists the selected model on create and update；`WorkbenchShell.spec.js` preselects the default model in the editor |
| 删除后搜索 | `desktop/src-tauri` `soft_deleted_prompt_is_hidden` |
| 按正文命中 | `desktop/src-tauri` `search_hits_content` |
| 复制后计数 | `desktop/src-tauri` `recording_use_increments_count`；`library.test.js` records last_used_at when a prompt is used |
