# 设置冲突策略诚实标明

> **给 Agent：** 不是新里程碑。立即同步的默认冲突已是较新 `updated_at`。设置不得把该策略写成尚未提供。不得假装用户已能选择保留本地。自动同步收藏与仅 Wi-Fi 仍可写尚未提供。

**Goal:** 同步页「冲突处理」标明较新者胜；未接通的同步行继续标明尚未提供。

**Architecture:** 只改设置文案与规格。冲突合并逻辑已在同步切片里，本计划不改推拉实现。

**Tech Stack:** 桌面设置弹窗、settings / sync 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 冲突行标明较新者胜

**Files:** `desktop/src/components/` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（冲突处理行含较新者胜，该行不含尚未提供）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 改冲突行文案；同步规格场景与测试映射**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [x] **Step 2: 关闭 done 记录；队列回到 deferred**
- [x] **Step 3: `./scripts/docs-check` 并提交**
