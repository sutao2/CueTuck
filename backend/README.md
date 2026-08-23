# PromptArk API

M5 广场合同的本仓库服务。只实现已在 [OpenAPI](../docs/reference/openapi/square.yaml) 里的路径。当前切片：邮箱密码会话。

```bash
cd backend
unset CARGO_TARGET_DIR
cargo test --locked
PROMPTARK_ALLOW_DEV_USER=1 cargo run
```

默认监听 `127.0.0.1:8787`，开发用户 `dev@promptark.local` / `devpass`。Postgres Testcontainers 测试在无 Docker 时保持 `ignore`。
