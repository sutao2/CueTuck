# 变更提案：客户端接 Google / GitHub

| 字段 | 值 |
|---|---|
| 状态 | 接受 |
| 日期 | 2026-08-25 |
| 影响规格 | [auth](../../specs/auth/spec.md) · [web](../../specs/web/spec.md) · [admin](../../specs/admin/spec.md) |

2026-08-25 人要求能做的都计划并做。后端 OAuth API 已在；客户端未接线。视为接受。决定仍以 [ADR 0013](../../architecture/decisions/0013-oauth-google-github.md) 为准。

## 为什么

登录弹窗只有邮箱密码。API 已能按已配置提供商跳转 Google / GitHub。浏览器收藏只提示「需要登录」，没有真正登录。

## 做什么

1. 登录弹窗在 `GET /v1/session/oauth/providers` 返回的提供商上显示按钮；未返回的不显示。
2. 桌面 OAuth 成功后 Refresh 进钥匙串。
3. 浏览器与管理台 OAuth / 邮箱登录成功后 Access 只留内存，Refresh 不进 Web Storage。
4. 浏览器已登录后收藏走 `PUT /v1/favorites/{id}`，不因此多一条内存库。

## 不做什么

- 云同步、账单、商店、QQ / LinuxDo、假凭据请求
- 改启动器去请求广场
- 声称生产或覆盖率已达门禁数字

## 依赖

预发后端持久化已完成。
