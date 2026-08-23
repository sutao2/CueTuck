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

Refresh token MUST 存放在系统钥匙串，MUST NOT 进入 Web Storage。Access 与 Refresh MUST 类型隔离并轮换。M5 提供者是邮箱 + 密码，见 [ADR 0008](../../architecture/decisions/0008-m5-backend-contract.md)。不绑定 QQ / LinuxDo / Google。

#### Scenario: 刷新轮换

- GIVEN 用户已登录并持有一对 Access 与 Refresh
- WHEN 使用 Refresh 换发新会话
- THEN 旧 Access 与旧 Refresh 均失效
- AND 新 Refresh 只进入系统钥匙串

## 测试映射

| 场景 | 测试 |
|---|---|
| 本地不要求账号 | 本地 CRUD 与启动器既有测试 |
| 发布触发 | `WorkbenchShell.spec.js` opens login from publish and resumes after success |
| 令牌 | `session.test.js` does not persist refresh in web storage；`desktop/src-tauri` `refresh_goes_to_store_access_does_not`；`backend` `create_session_isolates_access_and_refresh` |
| 刷新轮换 | 未开始 |
| 合同登录 | `squareContract.test.js` `POST /v1/session` |
