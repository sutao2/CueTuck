# 自动同步收藏与发布草稿

> **给 Agent：** 不是新里程碑。收藏与发布在线路径已接通。开启本开关后，断网的收藏/取消收藏与发布快照写入本机队列，联网后送出。不得假装已经到达服务器。不得写成本地副本。不得声称商店上架。

**Goal:** 设置可开关「自动同步收藏与发布草稿」；打开且已登录时，收藏或发布失败则入队；立即同步或下次成功请求时冲刷；开关关闭则不入队。

**Architecture:** 本机设置 `auto_sync_queue`。队列 JSON 存在 `sync_queue`。每条带当前账号邮箱，冲刷只送本账号。同一收藏 id 后写覆盖；同一发布源后写覆盖。

**Tech Stack:** 桌面设置、syncQueue、square 客户端、settings / square / publish 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 断网收藏与发布入队并冲刷

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/settings/spec.md` · `docs/specs/square/spec.md` · `docs/specs/publish/spec.md`

- [x] **Step 1: Write the failing test**（开关打开时失败的收藏/发布入队且不写本地副本；冲刷后到达服务端；关闭开关不入队；设置行不再写尚未提供）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 本机队列、设置开关、工作台走入队与立即同步冲刷**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
