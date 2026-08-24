# 浏览器接预发广场

> **给 Agent：** 不是新里程碑。`web/` 用本仓库预发 API 浏览/下载到内存库。不要写桌面 SQLite。不要假装已与桌面同步。启动器与 MCP 仍不请求广场。

**Goal:** 浏览器广场在 backend 开启时能列出预发条目并下载进当前标签页；backend 未开时给非阻断离线说明并可回本地。

**Architecture:** `fetch('http://127.0.0.1:8787/v1/square/items')`。下载走已有 `/content`，写入 `memoryLibrary`，`source` 标成 downloaded。CORS 允许 `http://localhost:5175`。收藏/发布若做，未登录必须说明原因，不得偷偷写内存库。

**Tech Stack:** Vue 3、Vitest（mock fetch）、Axum CORS。

## Global Constraints

- 浏览器下载只进标签页内存，不得声称写入桌面 SQLite。
- 启动器与 MCP 不请求广场或管理接口。
- `backend/` 仍是预发。
- 没有本计划之外的应用文件。

---

### Task 0: 规格场景

**Files:** `docs/specs/web/spec.md` · 本计划

- [x] **Step 1: 增加浏览器广场离线说明、匿名下载进内存库、不声称桌面已同步；映射先写「未开始」**
- [x] **Step 2: `./scripts/docs-check`**

### Task 1: CORS

**Files:** `backend/src/lib.rs`

- [x] **Step 1: Write the failing test**（Origin `http://localhost:5175` 的 OPTIONS 或 GET 带 ACAO）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: `PREVIEW_ORIGINS` 加入 5175（localhost 与 127.0.0.1）。不要对任意 Origin 放行**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check`**

### Task 2: 列表与离线

**Files:** `web/src/square.js` · `web/src/WebApp.vue` · `web/src/WebApp.spec.js`

- [x] **Step 1: Write the failing test**（fetch 失败时广场显示离线说明且可回本地；成功时列表出现标题）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 广场空间拉预发列表；失败不阻断切回本地**
- [x] **Step 4: 测试 PASS；映射「浏览器广场离线」**
- [x] **Step 5: `./scripts/docs-check`**

### Task 3: 下载进内存

**Files:** `web/src/square.js` · `web/src/memoryLibrary.js` · `web/src/WebApp.spec.js`

- [x] **Step 1: Write the failing test**（点下载后内存列表有该标题；说明仍写尚未与桌面同步；不出现「已写入本机 SQLite」）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 匿名 GET content，写入内存库。收藏未登录不得新增内存副本**
- [x] **Step 4: 测试 PASS；映射「浏览器匿名下载」**
- [x] **Step 5: `./scripts/docs-check`**
