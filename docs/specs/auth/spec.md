# 认证

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M5 实现 |
| 第一期 | M0–M4 不实现登录 |

## Purpose

为收藏、发布和云同步提供账号。第一期以本机访客身份工作。

## Requirements

### Requirement: 第一期无登录

第一期 MUST NOT 打开 OAuth，MUST NOT 把 refresh token 写入任何存储。用户可以完整使用本地库与启动器。

#### Scenario: 本地不要求账号

- GIVEN 全新安装且从未登录
- WHEN 用户创建提示词并用启动器复制
- THEN 操作成功
- AND 不出现登录门闩

### Requirement: 触发登录的动作（M5）

收藏与发布 MUST 在未登录时打开登录，并写明触发原因。下载 MUST 不打开登录。

#### Scenario: 发布触发

- GIVEN 用户未登录（M5）
- WHEN 用户确认发布
- THEN 登录界面说明「发布需要登录」
- AND 登录成功后恢复发布流程

### Requirement: 令牌（M5）

Refresh token MUST 存放在系统钥匙串，MUST NOT 进入 Web Storage。Access 与 Refresh MUST 类型隔离并轮换。M5 提供者是邮箱 + 密码，见 [ADR 0008](../../architecture/decisions/0008-m5-backend-contract.md)。Google 与 GitHub 授权码登录见 [ADR 0013](../../architecture/decisions/0013-oauth-google-github.md)。不绑定 QQ / LinuxDo。

#### Scenario: 刷新轮换

- GIVEN 用户已登录并持有一对 Access 与 Refresh
- WHEN 使用 Refresh 换发新会话
- THEN 旧 Access 与旧 Refresh 均失效
- AND 新 Refresh 只进入系统钥匙串

### Requirement: 口令存储

新写入的口令校验器 MUST 为 Argon2id。MUST NOT 把裸 SHA-256 hex 当作口令存储。

#### Scenario: 口令 KDF

- GIVEN 新账号口令
- WHEN 写入存储
- THEN 校验器是 Argon2id PHC 字符串
- AND 同一口令可以校验通过
- AND 该口令的 SHA-256 hex 不得当已存储校验器通过

### Requirement: 进程重启后仍在

当 API 配置了 Postgres，账号、令牌、收藏、投稿、广场条目与 `square_public` MUST 在进程重启后仍可用。

#### Scenario: 进程重启后会话

- GIVEN 用户已在 Postgres 存储上登录
- WHEN 使用同一数据库再开一个进程态
- THEN 仍可用原 Refresh 轮换出新会话

### Requirement: Google 与 GitHub

系统 MUST 在凭据已配置时提供 Google 与 GitHub 授权码登录。未配置的提供商 MUST 不可用。不得发假的提供商请求。

#### Scenario: 已配置则跳转授权

- GIVEN Google 客户端凭据已配置
- WHEN 请求 `GET /v1/session/oauth/google`
- THEN 302 到 Google 授权地址且含 client_id

#### Scenario: 回调签发会话

- GIVEN 有效授权码与 state
- WHEN 请求回调
- THEN 返回 Access 与 Refresh

## 测试映射

| 场景 | 测试 |
|---|---|
| 本地不要求账号 | 本地 CRUD 与启动器既有测试 |
| 发布触发 | `WorkbenchShell.spec.js` opens login from publish and resumes after success |
| 令牌 | `session.test.js` does not persist refresh in web storage；`desktop/src-tauri` `refresh_goes_to_store_access_does_not`；`backend` `create_session_isolates_access_and_refresh` |
| 刷新轮换 | `session.test.js` rotates access without writing refresh to web storage；`desktop/src-tauri` `rotate_replaces_refresh_in_store`；`backend` `refresh_rotates_and_invalidates_old_pair` |
| 合同登录 | `squareContract.test.js` `POST /v1/session` |
| 口令 KDF | `backend` `password_uses_argon2id_not_sha256` |
| 进程重启后会话 | `backend` `session_survives_new_appstate_on_postgres` |
| 已配置则跳转授权 | `backend` `oauth_google_redirects_when_configured` |
| 回调签发会话 | `backend` `oauth_callback_with_mock_code_issues_session` |
