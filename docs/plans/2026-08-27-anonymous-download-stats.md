# 匿名下载统计

> **给 Agent：** 不是新里程碑。广场匿名下载已接通。本开关默认关闭；打开后，成功下载只上报条目 id，不含账号、正文或标题。关闭时不请求。不得把 GET 正文当成统计。统计失败不得阻断下载。不得声称商店上架。手动配置代理仍可写跟随系统。

**Goal:** 设置可开关「匿名下载统计」；打开且本地下载成功后 `POST /v1/square/items/{id}/downloads`（无 Authorization）；默认关闭不静默上报。热门排序按次数降序。

**Architecture:** 本机设置 `anonymous_download_stats`。计数在服务端内存表与 Postgres `download_count`，不记谁下载。桌面在下载成功后 ping；Tauri 走 invoke，避免 CSP 拦 fetch。

**Tech Stack:** 桌面设置、square 客户端、Tauri square 命令、backend 广场、OpenAPI、settings / square 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 开关打开才上报条目 id

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `desktop/src-tauri/src/commands/square.rs` · `backend/src/` · `docs/reference/openapi/square.yaml` · `docs/specs/settings/spec.md` · `docs/specs/square/spec.md`

- [x] **Step 1: Write the failing test**（打开后成功下载 POST 不含 Authorization；关闭不请求；统计失败仍下载成功；GET 正文不加次数；未知 id 404；设置行不再写尚未提供）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 本机开关、下载后 ping、匿名 POST、热门按次数、Postgres 列**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
