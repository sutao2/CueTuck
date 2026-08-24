# 预发后端持久化

> **给 Agent：** 不是新里程碑。把已有 OpenAPI 接到本机已启动的 Postgres / Redis / MinIO。独立库 `promptark`，不写 Flyway 库 `pl`。Google / GitHub 走 `/v1/session/oauth`。不要接账单或云同步。启动器与 MCP 仍不请求广场。

**Goal:** `cargo run` 连上本机 `promptark` 库后，重启进程仍能登录、列出广场、审核投稿、读写收藏；口令以 Argon2id 存储；Google / GitHub 授权码可用；健康检查能看到 Postgres / Redis / MinIO。

**Architecture:** 路由仍是 `/v1/`。单测走内存。运行时连 `postgres://pl:pl@127.0.0.1:5432/promptark`。表不对就 DROP 重建。

**Tech Stack:** Axum、sqlx、Argon2id、Redis、MinIO、本机已有 Docker 服务。

## Global Constraints

- 本地优先。启动器与 MCP 不请求广场。
- `backend/` 仍是预发，不声称生产或上架。
- 不对接旧 Spring 路径，不把 `pl` 当现行库。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: Argon2id

**Files:** `backend/src/lib.rs` · `backend/Cargo.toml` · `docs/specs/auth/spec.md`

- [x] **Step 1: Write the failing test**（新哈希不是 64 位 hex；同一口令可校验；SHA-256 hex 不得当已存储校验器通过）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: `hash_password` 改为 Argon2id PHC；登录用 `verify`**
- [x] **Step 4: 测试 PASS；映射「口令 KDF」**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 本机库与 schema

**Files:** `backend/docker-compose.yml` · `backend/schema.sql` · `backend/.env.example` · `backend/README.md`

- [x] **Step 1: 写 compose（`promptark` / 端口 5433）与 `CREATE TABLE IF NOT EXISTS` schema**
- [x] **Step 2: 启动 Docker Desktop（若 daemon 未起）并 `docker compose up -d`；确认 `SELECT 1`**
- [x] **Step 3: README / `.env.example` 写明 URL；写明禁止指向旧库 `pl`**
- [x] **Step 4: `./scripts/docs-check` 并提交**

### Task 3: 账号与会话进 Postgres

**Files:** `backend/src/lib.rs` · `backend/src/main.rs` · `backend/Cargo.toml`

- [x] **Step 1: Write the failing test**（去掉 `session_api_postgres_container` 的 `ignore` 与 panic：对容器内库登录成功；进程内换一个 `AppState` 仍能用同一 Refresh 轮换）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: sqlx 读写 `accounts` / `access_tokens` / `refresh_tokens`；`main` 有 URL 则连库并种子账号**
- [x] **Step 4: 测试 PASS；映射「进程重启后会话」**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 4: 广场、投稿、收藏、设置进 Postgres

**Files:** `backend/src/lib.rs`

- [x] **Step 1: Write the failing test**（同一 Postgres：投稿带快照并 approve 后列表可见；收藏仍在；关闭 `square_public` 后匿名列表失败或空；换 `AppState` 仍成立）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 四张业务表走 sqlx；内存实现保持现有单测绿**
- [x] **Step 4: 测试 PASS；映射广场/发布/管理「重启仍在」**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 5: 本机说明书

**Files:** `docs/how-to/local-dev.md` · `docs/architecture/overview.md` · `docs/plans/status.md` · 本计划勾选

- [x] **Step 1: local-dev 写 compose 与 URL；overview 写预发存 Postgres**
- [x] **Step 2: 无 URL 的 `cargo test --locked` 仍全绿；有 Docker 时容器测试绿**
- [x] **Step 3: `./scripts/docs-check`；更新 status；提交**
