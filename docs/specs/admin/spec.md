# 管理台

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M6 目标 |
| 第一期 | M0–M4 无管理端；M5 不实现审核写路径 |
| 关联 | [发布](../publish/spec.md) · [认证](../auth/spec.md) · [M6](../../plans/milestones/m6.md) |

## Purpose

给运营人员在独立浏览器里审核广场投稿、查看用户、改运行时设置。不进桌面安装包，不改作者本机 SQLite。

## Requirements

### Requirement: 独立于桌面

管理端 MUST 是独立 Web 应用。桌面安装包 MUST NOT 包含管理源码或管理路由。启动器 MUST NOT 请求管理接口。

#### Scenario: 桌面包不含管理

- GIVEN 构建桌面客户端
- WHEN 检查安装包与 `desktop/` 依赖
- THEN 产物不含 `admin-web` 源码
- AND 不含管理审核路由

#### Scenario: 启动器不请求管理

- GIVEN 启动器窗口已打开
- WHEN 用户搜索或选择本地条目
- THEN 不发起管理接口请求

### Requirement: 管理员身份

管理写路径 MUST 要求带管理员角色的 Access。普通用户 Access MUST 被拒绝且不改发布状态。管理端 Refresh MUST NOT 进入 Web Storage。

#### Scenario: 普通令牌不能审核

- GIVEN 一条 `pending` 发布
- AND 调用方持有无管理员角色的 Access
- WHEN 请求通过或驳回
- THEN 请求失败
- AND 该条仍为 `pending`

#### Scenario: 管理端不持久化 Refresh

- GIVEN 运营人员在浏览器管理端登录成功
- WHEN 检查 Web Storage
- THEN 不出现 Refresh
- AND 关闭标签后须重新登录

#### Scenario: 查询管理员身份

- GIVEN 调用方持有管理员 Access
- WHEN 请求 `GET /v1/admin/me`
- THEN 返回该账号邮箱与管理员角色

- GIVEN 调用方持有普通用户 Access
- WHEN 请求 `GET /v1/admin/me`
- THEN 请求失败

### Requirement: 审核发布

管理员 MUST 能列出待审发布，并能通过或驳回。审核结果 MUST NOT 锁定或改写作者本地正文。

#### Scenario: 列出待审

- GIVEN 至少一条 `pending` 发布
- WHEN 管理员打开审核列表
- THEN 能看到该条的标识与来源摘要

#### Scenario: 通过不改本地

- GIVEN 一条 `pending` 发布，作者本地仍可编辑
- WHEN 管理员将其标为通过
- THEN 远端状态不再是 `pending`
- AND 作者本地该条仍可编辑且正文未被远端覆盖

#### Scenario: 驳回不删本地

- GIVEN 一条 `pending` 发布
- WHEN 管理员将其驳回
- THEN 远端状态为驳回
- AND 作者本地该条仍在库中

### Requirement: 用户只读列表

管理端 MUST 能列出已注册用户的邮箱与角色。本规格 MUST NOT 提供改密、删除账号或绑定第三方登录。

#### Scenario: 看到邮箱与角色

- GIVEN 库中有管理员与普通用户
- WHEN 管理员打开用户页
- THEN 列表含各自邮箱与角色
- AND 没有改密或删除控件

### Requirement: 运行时设置

管理员 MUST 能改一项已文档化的运行时开关，且对后续广场请求生效。未登录浏览广场 MUST NOT 依赖打开管理端页面。

#### Scenario: 关闭公开广场

- GIVEN 运行时开关允许匿名浏览广场
- WHEN 管理员关闭该开关并保存
- THEN 后续匿名广场列表请求失败或返回空
- AND 本地库与启动器仍全部可用

## 测试映射

| 场景 | 测试 |
|---|---|
| 管理合同 | `adminContract.test.js` lists `/v1/admin` paths with admin auth |
| 桌面包不含管理 | `desktop` `packageIsolation.test.js` does not depend on or bundle admin-web |
| 启动器不请求管理 | `LauncherApp.spec.js` does not request admin APIs while searching locally |
| 普通令牌不能审核 | `backend` `regular_token_cannot_review_publication` |
| 管理端不持久化 Refresh | `admin-web` `session.test.js` does not persist refresh in web storage |
| 查询管理员身份 | 未开始 |
| 列出待审 | `backend` `admin_lists_pending_and_can_approve`；`admin-web` `AdminApp.spec.js` lists pending after login |
| 通过不改本地 | `backend` `admin_lists_pending_and_can_approve` 将远端标为 approved；本切片不写桌面库 |
| 驳回不删本地 | `backend` `admin_rejects_publication` 将远端标为 rejected；本切片不写桌面库 |
| 看到邮箱与角色 | `backend` `admin_lists_user_emails_and_roles`；`admin-web` `AdminApp.spec.js` lists emails and roles without password or delete controls |
| 关闭公开广场 | `backend` `admin_can_close_public_square`；`admin-web` `AdminApp.spec.js` saves the anonymous square setting |
