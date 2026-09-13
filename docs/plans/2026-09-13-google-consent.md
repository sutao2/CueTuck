# Google 登录显式账号选择与授权确认

- 日期：2026-09-13
- 状态：服务端修正及运行验证完成
- 问题：用户反馈点击 Google 后未见授权确认即登录。当前授权请求未传 prompt；沿用旧版 Client ID 时，Google 可能复用已有登录和授权。

## 切片与验收

1. Google 普通登录授权请求增加 `prompt=select_account consent`，每次请求账号选择和确认；GitHub 不加 Google 特定参数。
2. 验证授权 URL 参数，以及只发起登录但无回调时轮询仍为 pending、不产生会话；保留既有授权码交换、邮箱验证和新旧回调安全校验。
3. 运行 OAuth 回归、文档检查，重启当前 8080 服务并验证实际授权跳转参数。无需重建未改动的客户端，不注销现有账号或代用户确认 Google 授权。

## Given / When / Then

- GIVEN 用户以前授权过同一 Google 应用；WHEN 再次点击 Google 登录；THEN 请求 Google 展示账号选择及 consent，不依靠默认的授权复用交互。
- GIVEN 客户端只打开了提供商网页；WHEN 尚无有效授权码回调而轮询；THEN 状态为 pending，没有登录会话。
- GIVEN 用户选择 GitHub；WHEN 发起登录；THEN 不附加 Google 特定 prompt，既有流程保持。

## 验收

- `cargo test --locked oauth -- --test-threads=4`：15 项通过，含新增 Google/GitHub 参数与无回调 pending 验证。
- `cargo build --locked` 成功；8080 服务已通过原本机启动脚本重启，数据库、Redis、MinIO 健康。
- 对运行服务检查两家 307 跳转：Google prompt 为 `select_account consent`，GitHub 不携带；两条新流程均未跟随外部授权页，轮询都为 pending，无会话。
- `./scripts/docs-check` 与 `git diff --check` 通过。未修改或重建客户端，未注销用户现有会话；实际 Google 账号选择与授权页面由用户重试验证。

规范依据：[Google OpenID Connect](https://developers.google.com/identity/openid-connect/openid-connect)。认证合同见 [认证规格](../specs/auth/spec.md)。实际用户此次 Google 页面未录制，因此原因是结合请求参数和官方行为的判断，不宣称已观测到提供商跳过确认。
