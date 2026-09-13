# CueTuck API

本仓库预发服务。广场合同见 [square.yaml](../docs/reference/openapi/square.yaml)；管理审核见 [admin.yaml](../docs/reference/openapi/admin.yaml)。当前切片：邮箱密码会话、广场列表与匿名正文、登录后提交审核（pending）、管理员审核 / 用户只读 / `square_public` 开关。浏览器预览（`:1420`）与管理台（`:5174`）靠 CORS 读本机 API。

```bash
cd backend
unset CARGO_TARGET_DIR
cargo test --locked
PROMPTARK_ALLOW_DEV_USER=1 PROMPTARK_BILLING_MOCK=1 cargo run
```

本机已配置凭据时使用 `./run-local` 启动：脚本读取工作目录中的 `.env` 后运行服务。`.env` 被 Git 忽略，含凭据时应设为 0600；不要把内容输出到日志或提交。直接 `cargo run` 不会自动读取该文件。Google/GitHub 环境变量沿用 `PROMPTARK_<提供商>_CLIENT_ID`、`CLIENT_SECRET`、`REDIRECT_URI`，数据库管理配置仍优先。

两家提供商控制台需登记与当前配置完全一致的回调。本机迁移现使用 `http://localhost:8080/api/v1/auth/oauth/callback`，旧路径与 `/v1/session/oauth/callback` 共用处理器；密钥接入与真实授权成功分别验收。本机 `./run-local` 从 `.env` 读取 `PROMPTARK_API_BIND=127.0.0.1:8080`，未提供配置时的默认端口保持 8787。

当前测试请保留 `PROMPTARK_BILLING_MOCK=1`；桌面设置的账号页和 Web 账单区将显示模拟操作。mock 的隔离与清空规则见 [账单规格](../docs/specs/billing/spec.md)。不设置该变量则保持原测试支付行为。

默认监听 `127.0.0.1:8787`。`cargo run` 连接本机 Postgres 库 `promptark`（不是 Flyway 库 `pl`）、Redis、MinIO。显式 `PROMPTARK_ALLOW_DEV_USER=1` 时才使用开发用户 `dev@promptark.local` / `devpass`（普通角色）与初始管理员 `admin@promptark.local` / `adminpass`；已有账号密码与角色不会被覆盖。非开发新实例须显式设置 `PROMPTARK_ADMIN_EMAIL` 和 `PROMPTARK_ADMIN_PASSWORD`，初始密码至少 12 字符。已有管理员的旧实例只记录初始化完成，不改账号。初始化完成后，改变这些环境变量不会重置密码或新增管理员；本人改密请使用管理台「账号安全」。Google / GitHub 优先读取管理台保存的配置，未保存时兼容 `PROMPTARK_GOOGLE_*` / `PROMPTARK_GITHUB_*` 与 `PL_*` 环境变量。表不对时可 `PROMPTARK_RESET_SCHEMA=1` 删表重建（会删除数据，常规启动不要设置）。

OAuth 配置密钥文件、备份及配置操作见[管理台 README](../admin-web/README.md)。默认测试账号仅用于本地开发，不应直接公开部署。

管理员初始化与安全迁移边界见[安全子切片计划](../docs/plans/2026-09-07-admin-security.md)。本次新增表/标记，不删除已有数据；不要为修复初始化错误删库，也不要直接回退至会覆盖密码的旧二进制。
# 认证限流

邮箱登录与管理端密码重新认证使用 60 秒窗口（全实例 300 次、同连接 IP 60 次、同账号 10 次），成功尝试也计入。Postgres 原子计数跨进程共享，重启不会清零；到期自动恢复。超限返回 429 和 `Retry-After: 60`，数据库故障返回 503，不降级绕过。后端不信任客户端自填的转发头；反向代理部署还须配置网关真实来源限流，不能以任意 X-Forwarded-For 覆盖连接 IP。
