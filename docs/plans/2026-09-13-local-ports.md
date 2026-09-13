# 本机服务端口与旧 OAuth 回调兼容

- 日期：2026-09-13
- 状态：本机迁移验收完成
- 用户授权：将占用 8080 的 xiangTA 迁到其他端口，让 PromptArk 使用 8080。

## 切片与验收

1. xiangTA 默认端口及本机上传地址迁至空闲 8081，同步 Flutter 调试连接和开发说明；运行配置测试，重启后检查 `/health`。不改用户既有文档工作区修改，不改账号或数据库。
2. PromptArk 增加旧回调路径别名，复用原有处理器及校验；新旧路径测试通过。配置本机后端监听 8080、两家 OAuth 回调沿用旧版本完整 URL；浏览器开发/构建环境和原生 Cargo 本机配置使用同一 origin。
3. 重建客户端、重启后端/管理端/客户端，检查两项目健康、授权 URL 对应源配置及回调处理。凭据不输出、不提交；真实外部账号授权仍单独验证。

## Given / When / Then

- GIVEN xiangTA 在 8080 运行；WHEN 执行授权迁移；THEN xiangTA 在 8081 健康，PromptArk 在 8080 健康，二者配置可在下次启动保留。
- GIVEN 提供商登记旧版回调 URL；WHEN 回调到 `/api/v1/auth/oauth/callback`；THEN 使用当前服务的 state、提供商和会话校验，和 `/v1/session/oauth/callback` 一致，无额外外部跳转。
- GIVEN 桌面或管理端连接 API；WHEN 完成本机重新构建/启动；THEN 请求使用 8080，不再连接已停止的 8787。

## 验收

- xiangTA：原 Maven 后端已正常退出并以 8081 重启，`/health` 返回 status=up；Flutter 配置测试 8 项通过。同步默认端口、上传地址、调试客户端及开发说明；原有用户文档修改未纳入提交。
- PromptArk：`cargo test --locked oauth -- --test-threads=4` 共 14 项通过；新旧回调均通过会话签发与无效 state 拒绝测试。`npm run build:local --prefix desktop` 成功（既有 9 条 Rust 警告）。
- 运行：API 8080 的 PostgreSQL/Redis/MinIO 全部健康；两家授权跳转的 Client ID 和完整 redirect_uri 与旧版本机配置完全一致，新旧回调均拒绝缺失 state（400）。未执行真实外部账号授权。
- 管理端：5174 重启后 Vite 实际注入 API origin 8080；客户端已重新打开，Cargo 编译依赖记录确认内置 origin 8080；8787 已停止。
- 本机 `.env`、前端各模式环境文件与原生 Cargo 配置确认被 Git 忽略，凭据未输出或提交。`./scripts/docs-check` 和差异空白检查通过。

认证合同见 [认证规格](../specs/auth/spec.md)，本机部署变量见 [部署说明](../../deploy/README.md)。
