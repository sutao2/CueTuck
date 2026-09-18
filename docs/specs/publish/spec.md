# 发布

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M5 实现 |
| 第一期 | M0–M4 本地主操作是「新建」不是「发布」 |

## Purpose

把本地提示词或合集提交到广场审核。发布不删除、不锁定本地内容。

## Requirements

### 单条提示词的公开附件

单条提示词可显式选择附件投稿，未选附件 MUST 不进入快照或公开读取。附件稿件 MUST 人工审核，不能以文本规则或模型代替文件检查。权限、不可变引用、预览与下架场景见 [P1c 计划](../../plans/2026-09-08-publication-assets.md)。合集成员附件的显式选择与关联见 [P1d 计划](../../plans/2026-09-08-collection-assets.md)。

### Requirement: 发布初筛与限额

服务端 MUST 在已启用自动审核时执行配置的日限额和公开快照初筛，不能依赖前端。关闭自动审核仍进入人工队列。新投稿记录实际结果与版本，旧投稿不得伪造初筛历史。本地规则自动通过须明确区分 AI 审核；任何必需检查不可用时转人工。

要求 AI 或稿件图片检查时，pending 与唯一审核任务同事务持久化，发布立即返回已接收；后台 worker 在事务外执行匹配的启用 Skill。主动投稿文本最多 32 KB，模型链路最多 25 秒；持久化租约、有限重试及状态读取遵循[管理规格](../admin/spec.md)。单模型失败可尝试主备，双模型一致/三模型多数必须有足够独立结果；各 Skill 均建议通过且最高风险分低于策略阈值、无本地待人工原因才允许自动上架。只有明确启用的视觉模型可接收经校验的选中图片，读取额外最多 25 秒；具体限制见[附件规格](../media/spec.md)。非图片文档与外部图片引用仍人工处理，含附件最终公开仍需人工确认。最终保存须复核策略/规则/模型与 Skill 版本，并让先到的人工决定优先。提供商失败或进程中断不得使已保存投稿丢失。

#### Scenario: 不能绕过检查

- GIVEN 用户直接请求 API，或同作者只剩一个日配额并发提交
- WHEN 创建投稿
- THEN 同样执行检查且最多一份占用剩余配额；超额返回 429，不创建投稿或改本地正文

### Requirement: 第一期无发布提交

M0–M4 MUST NOT 出现可成功提交审核的发布动作。M5 起以「选择本地源」与「审核与本地并行」为准。

### Requirement: 选择本地源（M5）

发布流程 MUST 能选择本地提示词或合集。未选择时 MUST 禁用提交。

加载中、读取失败与空库 MUST 明确区分，失败提供读取重试；关闭再打开后，旧请求不得更改页面。提交期间禁止重复操作，失败保留选择；成功与本机入队使用离开发布页后仍可见的提示。已提交成功但队列后处理失败 MUST 明确说明已提交，不得引导重复投稿。具体场景见 [P3c 计划](../../plans/2026-09-08-publish-sync-feedback.md)。

#### Scenario: 未选源

- GIVEN 发布页面打开且未选中本地内容
- WHEN 查看提交按钮
- THEN 按钮不可用

### Requirement: 审核与本地并行（M5）

提交后本地内容 MUST 仍可编辑。远端审核状态 MUST 不覆盖未发布的本地正文。

#### Scenario: 审核与本地并行

- GIVEN 用户已提交一条本地提示词审核
- WHEN 用户编辑该条本地正文并保存
- THEN 本地列表显示新正文
- AND 编辑器不被禁用

### Requirement: 发布草稿队列

开启「自动同步收藏与发布草稿」且已登录时，发布提交失败 MUST 把标题与正文快照写入本机队列，MUST NOT 改写本地正文，MUST NOT 假装已经到达审核。立即同步 MUST 把本账号队列送出。

#### Scenario: 断网发布入队

- GIVEN 用户已登录且开关已打开，发布请求失败
- WHEN 用户提交一条本地提示词
- THEN 本机队列含该快照
- AND 本地正文不变

#### Scenario: 冲刷发布到达服务端

- GIVEN 队列中有一条发布快照且网络已恢复
- WHEN 冲刷队列
- THEN 审核记录被创建
- AND 队列清空

#### Scenario: 新稿成功后清除旧草稿

- GIVEN 同一本地源的旧快照还在离线队列
- WHEN 新快照直接提交成功
- THEN 旧快照从队列清除，不再重复提交审核
- AND 仅入队时界面明确说明尚未提交审核

### Requirement: 提交快照与通过后上架

`POST /v1/publications` MUST 能带上标题与正文快照。管理员通过后，该快照 MUST 出现在广场列表。MUST NOT 用审核结果覆盖本地正文。缺快照时 MUST NOT 把该条假装已上架。

#### Scenario: 通过后进广场列表

- GIVEN 已登录提交标题为「新稿」的快照
- WHEN 管理员通过该审核
- THEN `GET /v1/square/items` 含「新稿」
- AND 本地库正文不被该操作改写

## 测试映射

### Requirement: 作者审核反馈

「我的发布」MUST 返回当前账号自己的审核历史与驳回原因，不暴露审核人员邮箱。客户端 MUST 展示原因及已有的审核时间，不修改本地正文。历史投稿没有事件时不得虚构原因或时间。

已上架投稿同时返回当前广场 visibility，客户端 MUST 区分审核通过和当前已下架/移入回收站，不能把审核通过一律当成仍在展示。

#### Scenario: 查看驳回原因

- GIVEN 自己的投稿被驳回并有审核原因
- WHEN 打开设置的「我的发布」
- THEN 能看见原因和审核时间，其他账号的投稿不出现
- AND 不显示审核人的私人邮箱，也不覆盖本地提示词

### Requirement: 合集成员快照

合集发布 MUST 传 `kind=collection` 及非空 `members` 数组，每项含标题、正文、系统分类、模型，以及可选的公开文件关联 `asset_ids`（仅引用稿件清单）。提交时读取当前成员，MUST NOT 上传本机路径、私有成员 ID 或使用记录。合集无成员、成员标题/正文为空、成员分类非法 MUST 拒绝。省略 kind 的旧请求仍按 prompt 处理。审核 MUST 展示成员正文；通过后保持合集类型与成员数量。成员快照不随本地后续编辑改变。合集封面采用独立媒体引用，规则见下方「合集封面发布」。

#### Scenario: 合集发布审核

- GIVEN 本地合集有两条不同分类和模型的提示词
- WHEN 提交后编辑本地成员，再由管理员通过审核
- THEN 广场为包含两条原始正文的合集，分类与模型保留
- AND 本地编辑结果不被审核改写

#### Scenario: 空或无效合集

- GIVEN 合集无成员或成员正文为空
- WHEN 提交审核
- THEN 明确失败，服务端不创建待审记录

### Requirement: 发布分类与模型快照

发布 MUST 保留所选远端分类与模型。发布窗口加载公共字典并允许选择广场分类/模型，默认匹配本地分类 ID；自定义小分类回退到仍启用的系统父类，无匹配项时默认未分类，不向广场发送私有分类 ID。后端 MUST 在写入事务内拒绝未知/停用分类及已知停用模型，历史无分类和自由文本模型保持兼容。远端字典不可用时显示提示并保留原有离线队列路径，不假报提交成功。

#### Scenario: 用户小分类发布

- GIVEN 本地「办公效率 / 周报」是自定义小分类
- WHEN 提交提示词审核
- THEN 快照分类为系统「办公效率」且保留模型
- AND 审核上架及正文下载返回同一分类与模型

| 场景 | 测试 |
|---|---|
| 点击发布 | M0–M4 无提交；M5 见「未选源」「审核与本地并行」 |
| 未选源 | `WorkbenchShell.spec.js` disables publish submit until a local source is selected |
| 审核与本地并行 | `WorkbenchShell.spec.js` keeps the local prompt editable after publish；`square.test.js` submits a publication without changing the local copy |
| 断网发布入队 | `syncQueue.test.js` queues a publication snapshot when auto-sync is on and the request fails |
| 冲刷发布到达服务端 | `syncQueue.test.js` flushes a queued publication when the transport recovers |
| 合同提交 | `squareContract.test.js` `POST /v1/publications`；`backend` `create_publication_requires_access_and_keeps_pending` |
| 通过后进广场列表 | `backend` `approve_with_snapshot_lists_on_square`；`approve_without_snapshot_does_not_list_on_square`；`square.test.js` submits a publication without changing the local copy |

投稿附件选择 MUST 展示本机图片预览、已选图片数及显式全选图片操作。存在图片但未选择时明确提示仅发布文字，不默认上传。审核通过后，广场分页封面和详情图集展示批准清单中的图片，不能只显示外部导入参考图。详见[投稿图片计划](../../plans/2026-09-14-publication-images.md)。

- Given 本机有图片但未勾选 When 提交 Then 明示只发布文字，不上传未选文件。
- Given 图片已选并通过审核 When 查看广场列表和详情 Then 首图成为封面，图集可打开原图；未通过或下架不得读取。

## 合集封面发布（2026-09-18）

- 有封面的合集提供「同时发布合集封面」勾选项，默认选中并展示封面及公开说明；取消后不上传封面。确认预览之前不上传；成员私有附件仍默认不选。
- `publications.cover` 为可空 JSONB，格式 `{layout: single|grid, asset_ids: [...]}`；单图最多 1 张、九宫格最多 9 张，按数组顺序显示。媒体引用复用 `asset_refs`，封面只接受 PNG/JPEG/GIF/WebP；未知、重复、与成员共享的引用及非合集附带封面均拒绝。
- 清单每项必须且只能关联封面或一个成员，总限额仍为 12 个、20 MiB、单文件 5 MiB。上传前一次校验；授权、哈希校验、人工审核和下架限制复用已有稿件附件流程。
- GIVEN 带封面稿件；WHEN 审核通过；THEN 分页广场返回有序 `cover_assets` 与布局，详情返回完整引用；下载验证所有字节后在导入事务中还原本地 cover_json，失败不留部分合集。
- GIVEN 旧稿件无 cover；WHEN 查看、审核或下载；THEN 保持原行为。旧已发布合集不会自动补传本机封面，需要重新发布并审核。
- 离线队列保存布局及已上传的引用，不保存封面本地路径或 base64；重试保留顺序。管理端单独标识合集封面，供审核者逐图查看。

### Requirement: 显式 AI 直接判定（替代强制人工公开边界）

启用 `ai_decides` 时，提示词、合集和 Skill 的完整 AI 结果可直接通过或拒绝；宽松判断与文件范围以 [AI 直接审核计划](../../plans/2026-09-18-ai-decisions.md) 为准，取代上文“一律人工审核”的要求。旧策略未启用时保持原规则；任何未完成检查、错误、版本变化或不明确结论不得假称通过或拒绝。图片传输范围升级为 PNG/JPEG/WebP 最多 12 张、总量 20 MiB，其他未支持格式仍转人工。技能脚本只按文本审查，绝不执行。人工处置和作者撤回优先，审核事件及审计须与结果同事务。
