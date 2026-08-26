# 网络页同步状态诚实标明

> **给 Agent：** 不是新里程碑。个人库立即同步已接通。网络页「同步状态」不得写成没有云同步。不得假装后台正在同步或已同步。自动同步收藏与仅 Wi-Fi 上传仍可写尚未提供。

**Goal:** 网络页同步状态标明手动立即同步；同步页 Wi-Fi 行不再写没有云同步引擎。

**Architecture:** 只改设置文案与规格。不改推拉实现，不接通后台自动同步。

**Tech Stack:** 桌面设置弹窗、settings 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 同步状态不再写没有云同步

**Files:** `desktop/src/components/` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（网络页同步状态不含没有云同步 / 尚未提供；不含已同步或正在同步）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 改网络页与 Wi-Fi 行文案；更新规格场景与测试映射**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
