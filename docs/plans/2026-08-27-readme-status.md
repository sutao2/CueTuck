# 仓库入口阶段诚实标明

> **给 Agent：** 不是新里程碑。M9 与完整产品切片已关闭。仓库根 README 与 CLAUDE 不得把已接通的同步、OAuth、更新、账单写成尚未接通。商店上架与生产托管仍不得假装接通。不得改应用行为。

**Goal:** README 标明 M9 已关闭且不把已接通云能力写成未接通；CLAUDE 不再把同步 / 更新 / 账单列为未接通示例；仍标明无商店包。

**Architecture:** 只改入口文案与文档规格。测试读仓库根文件。

**Tech Stack:** README、CLAUDE、documentation 规格、桌面隔离测试。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 入口不把已接通写成未接通

**Files:** `README.md` · `CLAUDE.md` · `desktop/src/platform/` · `docs/specs/documentation/spec.md`

- [x] **Step 1: Write the failing test**（README 含 M9 已关闭，不含把云同步、OAuth、自动更新写成不得假装接通；仍无商店包）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 改 README 与 CLAUDE；更新规格场景与测试映射**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/documentation/spec.md`

- [x] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [x] **Step 2: 关闭 done 记录；队列回到 deferred**
- [x] **Step 3: `./scripts/docs-check` 并提交**
