# 认证

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M5 实现 |
| 第一期 | M0–M4 不实现登录 |

## Purpose

为收藏、发布和云同步提供账号。第一期以本机访客身份工作。

## Requirements

### Requirement: 邮箱身份验证与注册门禁

新邮箱注册 MUST 在邮件验证后创建 user，不得匿名创建管理员。站点注册关闭时邮箱与新 OAuth 账号创建都拒绝，已有账号不受注册开关影响。发起注册/重置使用统一非枚举响应和频控；验证码随机、仅摘要索引、30 分钟有效、一次性，正文不进入日志或浏览器持久存储。

#### Scenario: 凭据过期或重复

- GIVEN 验证凭据已使用、过期、撤销或被同目的重发替代
- WHEN 注册、接受邀请或重置密码
- THEN 不修改账号并返回统一无效凭据错误
- AND 成功时凭据消耗、账号写入、会话撤销与审计同事务；失败回滚全部

#### Scenario: 邀请不能成为匿名提权入口

- GIVEN owner 邀请新邮箱成为 admin/reviewer
- WHEN 用户以邀请凭据设置密码，或邀请者已撤权/目标已有账号
- THEN 仅有效邀请可创建指定角色；不覆盖已有账号，不邀请 owner，不自动签发会话

### Requirement: 清晰的登录表单

#### Scenario: 已登录账号入口

- GIVEN 客户端已有登录会话
- WHEN 点击侧栏账号入口
- THEN 直接显示设置中的「账号与广场」、当前邮箱和退出操作，不再次显示登录表单
- AND 退出进行中防止重复请求，失败显示错误；成功回到未登录状态并清除旧账号收藏显示，本地内容不变

登录 MUST 使用紧凑单列、可见邮箱与密码标签和整行主按钮；入口原因不得重复标题，技术存储说明处于次要层级。邮箱提交支持 Enter，并在进行中禁用重复请求，失败保留输入并允许重试；邮箱等待与浏览器 OAuth 授权等待必须区分。邮箱请求进行中不离开登录，OAuth 保留取消授权的既有行为。桌面采用工作区页面，返回来源与焦点规则见[工作台规格](../workbench/spec.md)。

#### Scenario: 提交反馈与重试

- GIVEN 用户已填写邮箱密码
- WHEN 提交尚未完成或请求失败
- THEN 进行中显示登录状态且不重复请求，失败显示错误并恢复可提交状态
- AND 表单不清空，不把邮箱登录显示成等待浏览器授权

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

提供商的管理端配置、启停及密钥保护见[管理台规格](../admin/spec.md)。账号关联 MUST 使用提供商已验证的邮箱；GitHub 使用已验证的主邮箱，Google 检查 `email_verified`，不得把未验证邮箱关联到已有管理员账号。

#### Scenario: 兼容旧版 OAuth 回调路径

- GIVEN 本机迁移沿用提供商已登记的旧回调地址
- WHEN 请求 `/api/v1/auth/oauth/callback`
- THEN 与 `/v1/session/oauth/callback` 使用同一处理器和 state、提供商、错误及会话校验，不重定向到其他进程或减弱认证
- AND 实际端口仍由部署配置决定，提供商授权请求的 redirect_uri 必须与登记地址一致

#### Scenario: 提供商邮箱校验

- GIVEN Google 未确认邮箱，或 GitHub 没有已验证的主邮箱
- WHEN 完成授权码交换并读取个人资料
- THEN 登录失败，不用该邮箱关联本地账号；GitHub 请求带应用 User-Agent
- AND 有已验证邮箱时继续原有会话流程

回归：`oauth::profile_tests::oauth_accounts_require_verified_provider_emails` 使用本机模拟提供商。

#### Scenario: 已配置则跳转授权

- GIVEN Google 客户端凭据已配置
- WHEN 请求 `GET /v1/session/oauth/google`
- THEN 302 到 Google 授权地址且含 client_id

#### Scenario: 回调签发会话

- GIVEN 有效授权码与 state
- WHEN 请求回调
- THEN 返回 Access 与 Refresh

#### Scenario: 登录界面列出已配置提供商

- GIVEN `GET /v1/session/oauth/providers` 返回 `google`
- WHEN 用户打开登录
- THEN 可见 Google 登录
- AND 不出现未返回的提供商
- AND 不出现 QQ / LinuxDo

#### Scenario: 未配置则只留邮箱密码

- GIVEN 提供商列表为空
- WHEN 用户打开登录
- THEN 仍可用邮箱密码
- AND 不出现 Google / GitHub 按钮

### Requirement: 安全初始化与事务会话

运行态 MUST 保留已有账号密码与角色，不在重启时覆盖。无管理员的新实例 MUST 使用服务器明确配置的初始凭据；默认开发账号仅在 `PROMPTARK_ALLOW_DEV_USER=1` 时插入且不覆盖已有账号。已初始化实例不重新创建管理员，同邮箱普通用户不得被初始化隐式提权。

#### Scenario: 初始化重试和重启

- GIVEN 新实例、已有管理员或已记录初始化完成的实例
- WHEN 首次启动、并发启动或修改服务器初始凭据后重启
- THEN 最多创建一位初始管理员；已有账号不改密码/角色；缺少必要初始化凭据时明确失败，不能开放匿名管理员注册

#### Scenario: 改密不能被并发旧会话绕过

- GIVEN 改密或全会话撤销与旧密码登录/Refresh 请求并发
- WHEN 请求完成
- THEN 已撤销令牌不能再换发会话；旧密码不能在改密完成后签发有效会话，失败事务不产生半对令牌
- AND Postgres 是授权来源，Redis 缓存不能使已撤销令牌重新有效

回归：初始化、会话与本人账号安全的隔离 Postgres 测试；详见[实施计划](../../plans/2026-09-07-admin-security.md)。

### Requirement: 所有者迁移与停用账号

新库初始化为 owner；旧库只能通过明确服务器配置一次选取现有未停用管理员，不能因重启重新提权或更改口令。角色规则见 [ADR 0018](../../architecture/decisions/0018-admin-fixed-roles.md)。

#### Scenario: 停用不能绕过

- GIVEN 账号已停用，包括已有 Access/Refresh 或关联 OAuth 身份
- WHEN 请求认证资源、密码登录、刷新或 OAuth 签发会话
- THEN 全部拒绝；重新启用也不能恢复先前已撤销令牌

#### Scenario: 一次所有者迁移

- GIVEN 旧库存在多个管理员
- WHEN 服务器明确选定一个邮箱后重复启动
- THEN 仅首次选定的有效管理员成为 owner，密码不变；普通用户邮箱被拒绝，迁移后不因环境变量再次提升其他账号

## 测试映射

### Requirement: 认证尝试节流

认证及管理密码重新校验 MUST 在密码运算前限制频率，限制规则与存储见[管理基础 1c](../../plans/2026-09-07-admin-navigation-throttle.md)。Postgres 运行态跨进程持久共享，不因重启清零；不信任任意转发头，不持久保存原邮箱、IP 或密码。

#### Scenario: 窗口限制和并发

- GIVEN 同一邮箱或来源在 60 秒内达到尝试额度
- WHEN 再次请求或并发请求，包含不存在账号与正确密码
- THEN 超额返回 429 和 Retry-After，不继续验证密码；窗口到期自动恢复，不能依赖改密码或重启解限

#### Scenario: 限流存储不可用

- GIVEN Postgres 认证限流读写失败
- WHEN 请求登录或高危重新认证
- THEN 返回服务不可用，不降级绕过限制或产生新会话

| 场景 | 测试 |
|---|---|
| 已登录账号入口 | `ClientPolish.spec.js` 账号设置分流/退出与失败反馈；真实 macOS 邮箱入口验证 |
| 提交反馈与重试 | `LoginModal.spec.js` 邮箱等待、防重复、失败保留输入与重试；浏览器隔离会话 Enter 提交 |
| 本地不要求账号 | 本地 CRUD 与启动器既有测试 |
| 发布触发 | `WorkbenchShell.spec.js` opens login from publish and resumes after success |
| 令牌 | `session.test.js` does not persist refresh in web storage；`desktop/src-tauri` `refresh_goes_to_store_access_does_not`；`backend` `create_session_isolates_access_and_refresh` |
| 刷新轮换 | `session.test.js` rotates access without writing refresh to web storage；`desktop/src-tauri` `rotate_replaces_refresh_in_store`；`backend` `refresh_rotates_and_invalidates_old_pair` |
| 合同登录 | `squareContract.test.js` `POST /v1/session` |
| 口令 KDF | `backend` `password_uses_argon2id_not_sha256` |
| 进程重启后会话 | `backend` `session_survives_new_appstate_on_postgres` |
| 已配置则跳转授权 | `backend` `oauth_google_redirects_when_configured` |
| 回调签发会话 | `backend` `oauth_callback_with_mock_code_issues_session` |
| 登录界面列出已配置提供商 | `WorkbenchShell.spec.js` shows google on login when providers include google |
| 未配置则只留邮箱密码 | `WorkbenchShell.spec.js` hides oauth buttons when providers empty |
