# 发布

| 字段 | 值 |
|---|---|
| 状态 | 实现中 |
| 第一期 | 本地主操作是「新建」不是「发布」 |

## Purpose

把本地提示词或合集提交到广场审核。发布不删除、不锁定本地内容。

## Requirements

### Requirement: 第一期无发布提交

M0–M4 MUST NOT 出现可成功提交审核的发布动作。M5 起以「选择本地源」与「审核与本地并行」为准。

### Requirement: 选择本地源（M5）

发布流程 MUST 能选择本地提示词或合集。未选择时 MUST 禁用提交。

#### Scenario: 未选源

- GIVEN 发布弹窗打开且未选中本地内容
- WHEN 查看提交按钮
- THEN 按钮不可用

### Requirement: 审核与本地并行（M5）

提交后本地内容 MUST 仍可编辑。远端审核状态 MUST 不覆盖未发布的本地正文。

#### Scenario: 审核与本地并行

- GIVEN 用户已提交一条本地提示词审核
- WHEN 用户编辑该条本地正文并保存
- THEN 本地列表显示新正文
- AND 编辑器不被禁用

## 测试映射

| 场景 | 测试 |
|---|---|
| 点击发布 | M0–M4 无提交；M5 见「未选源」「审核与本地并行」 |
| 未选源 | `WorkbenchShell.spec.js` disables publish submit until a local source is selected |
| 审核与本地并行 | `WorkbenchShell.spec.js` keeps the local prompt editable after publish；`square.test.js` submits a publication without changing the local copy |
| 合同提交 | `squareContract.test.js` `POST /v1/publications`；`backend` `create_publication_requires_access_and_keeps_pending` |
