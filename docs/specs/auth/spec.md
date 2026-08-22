# 认证

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M5 实现 |
| 第一期 | 不实现登录 |

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

Refresh token MUST 存放在系统钥匙串，MUST NOT 进入 Web Storage。Access 与 Refresh MUST 类型隔离并轮换。具体提供者在 M5 变更中选定，第一期不绑定 QQ / LinuxDo / Google。
