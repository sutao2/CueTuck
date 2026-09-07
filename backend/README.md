# PromptArk API

本仓库预发服务。广场合同见 [square.yaml](../docs/reference/openapi/square.yaml)；管理审核见 [admin.yaml](../docs/reference/openapi/admin.yaml)。当前切片：邮箱密码会话、广场列表与匿名正文、登录后提交审核（pending）、管理员审核 / 用户只读 / `square_public` 开关。浏览器预览（`:1420`）与管理台（`:5174`）靠 CORS 读本机 API。

```bash
cd backend
unset CARGO_TARGET_DIR
cargo test --locked
PROMPTARK_ALLOW_DEV_USER=1 PROMPTARK_BILLING_MOCK=1 cargo run
```

当前测试请保留 `PROMPTARK_BILLING_MOCK=1`；桌面设置的账号页和 Web 账单区将显示模拟操作。mock 的隔离与清空规则见 [账单规格](../docs/specs/billing/spec.md)。不设置该变量则保持原测试支付行为。

默认监听 `127.0.0.1:8787`。`cargo run` 连接本机 Postgres 库 `promptark`（不是 Flyway 库 `pl`）、Redis、MinIO。开发用户 `dev@promptark.local` / `devpass`（普通角色）。管理员 `admin@promptark.local` / `adminpass`。Google / GitHub 优先读取管理台保存的配置，未保存时兼容 `PROMPTARK_GOOGLE_*` / `PROMPTARK_GITHUB_*` 与 `PL_*` 环境变量。表不对时可 `PROMPTARK_RESET_SCHEMA=1` 删表重建（会删除数据，常规启动不要设置）。

OAuth 配置密钥文件、备份及配置操作见[管理台 README](../admin-web/README.md)。默认测试账号仅用于本地开发，不应直接公开部署。
