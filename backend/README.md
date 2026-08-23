# PromptArk API

本仓库预发服务。广场合同见 [square.yaml](../docs/reference/openapi/square.yaml)；管理审核见 [admin.yaml](../docs/reference/openapi/admin.yaml)。当前切片：邮箱密码会话、广场列表与匿名正文、登录后提交审核（pending）、管理员列出/通过/驳回。浏览器预览（`:1420`）与管理台（`:5174`）靠 CORS 读本机 API。

```bash
cd backend
unset CARGO_TARGET_DIR
cargo test --locked
PROMPTARK_ALLOW_DEV_USER=1 cargo run
```

默认监听 `127.0.0.1:8787`。开发用户 `dev@promptark.local` / `devpass`（普通角色）。管理员 `admin@promptark.local` / `adminpass`。Postgres Testcontainers 测试在无 Docker 时保持 `ignore`。
