# 仅 Wi-Fi 下同步图片

> **给 Agent：** 不是新里程碑。立即同步已接通。开启本开关且无法判定为 Wi-Fi 时跳过合集封面，仍同步标题与正文。不得把远端已有封面用空封面覆盖掉。不得声称商店上架。自动同步收藏队列仍可标明尚未提供。

**Goal:** 设置可开关「仅 Wi-Fi 下同步图片」；打开且网络不是 Wi-Fi 时立即同步不推送本机封面，提示词正文与合集标题仍推送。

**Architecture:** 本机设置 `sync_wifi_images`。推送前若需跳过图片则先拉远端，合集封面沿用远端已有值，没有远端则空封面。网络类型可注入；未注入且无法判定时视为非 Wi-Fi。

**Tech Stack:** 桌面设置、librarySync、settings / sync 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 非 Wi-Fi 跳过封面仍同步正文

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/sync/spec.md` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（开关打开且非 Wi-Fi 时合集封面不进 PUT；标题与提示词正文仍进；Wi-Fi 或开关关闭仍推封面；设置行不再写尚未提供）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 本机设置、网络判定与推送跳过封面；设置页提供开关**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/sync/spec.md`

- [x] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [x] **Step 2: 关闭 done 记录；队列回到 deferred**
- [x] **Step 3: `./scripts/docs-check` 并提交**
