# 设计：预发后端持久化

## 方案

Axum `/v1/` 不变。无数据库 URL 时单测走内存。`cargo run` 连本机已有服务：

| 服务 | 默认 |
|---|---|
| Postgres | `postgres://pl:pl@127.0.0.1:5432/promptark`（**不是** 库 `pl`） |
| Redis | `redis://127.0.0.1:6379` |
| MinIO | `http://127.0.0.1:9000`，bucket `prompt-launcher-media`，密钥与旧 compose 相同 |

表不对时 `DROP TABLE ... CASCADE` 后按 `backend/schema.sql` 重建。OAuth 授权码、state HMAC、web_message / browser 轮询对齐旧实现，path 换成 `/v1/session/oauth/...`。凭据读 `PL_GOOGLE_*` / `PL_GITHUB_*` 或 `PROMPTARK_*`。

## 文件

`backend/schema.sql`、`backend/src/{password,postgres,oauth,media}.rs`、`backend/src/lib.rs`、`backend/src/main.rs`、OpenAPI、local-dev。

## 风险

- 禁止往库 `pl` 的 Flyway 表写 PromptArk 行。
- Argon2 单测用弱参数，运行时用默认参数。
- MinIO / Redis 挂掉时媒体或浏览器 OAuth 轮询失败，邮箱登录与广场读仍应可用（令牌在 Postgres）。
