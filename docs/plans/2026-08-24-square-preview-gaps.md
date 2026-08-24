# 广场预发缺口

> **给 Agent：** 不是新里程碑。只补已有广场/发布合同在预发 backend 上的缺口。不要接 Postgres。不要声称生产排序。启动器仍不请求广场。

**Goal:** 预发广场能按合同返回条目详情；推荐/最新/热门可区分；已登录收藏排序可用；审核通过后该条出现在广场列表。

**Architecture:** 仍用 `backend/` 内存。`GET /v1/square/items/:id` 补上 OpenAPI 已写的详情。发布时带标题与正文快照，通过审核后写入 `items`，不改桌面 SQLite。排序只做预发可测差异，不发明真实热度算法。

**Tech Stack:** Axum、Vitest、现有 `squareContract.test.js`。

## Global Constraints

- 本地优先。启动器与 MCP 不请求广场。
- `backend/` 仍是预发，不声称生产、不声称上架。
- 审核通过不得锁定或改写本地正文。
- 没有本计划之外的应用文件。

---

### Task 0: 规格场景

**Files:** `docs/specs/square/spec.md` · `docs/specs/publish/spec.md` · 本计划

- [ ] **Step 1: 广场规格增加详情、排序/模型筛选、已登录收藏排序场景；映射先写「未开始」**
- [ ] **Step 2: 发布规格增加「提交带快照、通过后进广场列表」；不覆盖本地正文**
- [ ] **Step 3: `./scripts/docs-check`**

### Task 1: 条目详情

**Files:** `backend/src/lib.rs` · `desktop/src/platform/squareContract.test.js`

- [ ] **Step 1: Write the failing test**（`serves_square_item_without_login`：`GET /v1/square/items/{id}` 匿名 200，缺 id 404）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 路由 `GET /v1/square/items/:id`，返回标题/类型/摘要，不含未公开字段**
- [ ] **Step 4: 测试 PASS；映射「条目详情」**
- [ ] **Step 5: `./scripts/docs-check`**

### Task 2: 预发排序与模型筛选

**Files:** `backend/src/lib.rs`

- [ ] **Step 1: Write the failing test**（同一批种子下 `sort=recommended|latest|hot` 顺序可区分；`model` 查询只留该模型）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 预发实现即可：推荐=种子序，最新=反转，热门=按标题；`q` 仍搜标题；增加 `model` 查询**
- [ ] **Step 4: 测试 PASS；映射「浏览排序与模型筛选」。不得写「真实热度」**
- [ ] **Step 5: `./scripts/docs-check`**

### Task 3: 已登录收藏排序

**Files:** `backend/src/lib.rs`

- [ ] **Step 1: Write the failing test**（未登录 `sort=favorites` 空列表；已登录返回该账号收藏）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 列表接口可选 Bearer；收藏排序读 `favorites` 表，不写本地库**
- [ ] **Step 4: 测试 PASS；映射「已登录收藏排序」**
- [ ] **Step 5: `./scripts/docs-check`**

### Task 4: 审核通过进入广场列表

**Files:** `backend/src/lib.rs` · `desktop/src/platform/square.js` · `desktop/src-tauri/src/commands/square.rs` · `docs/reference/openapi/square.yaml`

- [ ] **Step 1: Write the failing test**（提交带 `title`/`content` 快照；approve 后 `GET /v1/square/items` 能见到该标题；本地库测试不得被改）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: `POST /v1/publications` 保存快照；`approve` 插入 `SquareItem`。桌面提交时带上当前标题与正文。缺快照不得假装已上架**
- [ ] **Step 4: 测试 PASS；映射「通过后进广场列表」「审核与本地并行」仍绿**
- [ ] **Step 5: `./scripts/docs-check`**
