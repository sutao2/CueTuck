# 个人库云同步

> **给 Agent：** 不是新里程碑。先读 [sync 规格](../specs/sync/spec.md) 与 [full-product design](../changes/full-product/design.md)。启动器与 MCP 仍只碰本机 SQLite。不要上架商店。不要声称打开了另一台机器的 SQLite 文件。

**Goal:** 登录后立即同步能把本机库与账号库对齐；浏览器登录后看到同一账号库；冲突默认较新者胜。

**Architecture:** `/v1/library/changes` 推拉。桌面写 SQLite 再同步。浏览器已登录走账号库。

**Tech Stack:** Axum、Postgres `promptark`、MinIO、 rusqlite。

## Global Constraints

- 本地优先。断网本机仍可用。
- 不得把 Refresh 写入 Web Storage。
- 不得声称打开了另一台机器的 SQLite 文件。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 变更推拉 API

**Files:** `backend/src/` · `docs/reference/openapi/square.yaml` · `docs/specs/sync/spec.md`

- [x] **Step 1: Write the failing test**（PUT 一条 prompt 后 GET `since=` 能拉到同一 id 与正文）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 表 `library_changes`；`GET/PUT /v1/library/changes`；需登录**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 桌面立即同步

**Files:** `desktop/src/components/SettingsModal.vue` · `desktop/src/platform/` · `docs/specs/sync/spec.md`

- [x] **Step 1: Write the failing test**（已登录点立即同步后账号库出现「本地仍在」；未登录打开登录且不出现已同步）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 登录后推拉本机 SQLite；未登录打开已有登录弹窗**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 3: 冲突与浏览器账号库

**Files:** `backend/src/` · `web/src/` · `docs/specs/sync/spec.md`

- [x] **Step 1: Write the failing test**（远端较新则本机正文为远端；浏览器登录后可见同一标题且不出现「已写入本机 SQLite」）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 默认 `updated_at` 较新者胜；浏览器已登录读写账号库**
- [x] **Step 4: 测试 PASS；关闭本计划 done 记录**
- [x] **Step 5: `./scripts/docs-check` 并提交**
