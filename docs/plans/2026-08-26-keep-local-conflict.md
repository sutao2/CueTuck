# 冲突时保留本地

> **给 Agent：** 不是新里程碑。默认仍是较新 `updated_at`。用户选择保留本地时不得覆盖该条本机正文。不得把未存在的本机条目拦掉。不得声称商店上架。

**Goal:** 设置可选较新者胜或保留本地；保留本地时立即同步不覆盖已有本机正文，远端独有条目仍可写入。

**Architecture:** 本机设置 `sync_conflict`。拉变更时若为 `keep_local` 且本地已有该 id 则跳过覆盖。推送仍走现有 PUT；后端较新者胜不变。

**Tech Stack:** 桌面设置、librarySync、settings / sync 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 保留本地不覆盖本机正文

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/sync/spec.md` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（选保留本地后远端更新更晚也不覆盖本机正文；设置可切换）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 本机设置与拉变更跳过；设置页提供选择**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/sync/spec.md`

- [x] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [x] **Step 2: 关闭 done 记录；队列回到 deferred**
- [x] **Step 3: `./scripts/docs-check` 并提交**
