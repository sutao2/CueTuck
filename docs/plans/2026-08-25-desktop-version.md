# 桌面包版本对齐

> **给 Agent：** 不是新里程碑。设置「当前版本」与检查更新比较的必须是本机构建版本。不得把 `package.json` 的占位 `0.0.0` 写成已安装版本。不得声称商店上架。

**Goal:** 桌面包 `package.json` 与 Cargo / `tauri.conf.json` 使用同一版本号，更新页展示该版本，已是最新时不把同号发行物当成可更新。

**Architecture:** 设置页与浏览器路径的 `checkForUpdates` 读 `package.json`；Tauri 路径读 `CARGO_PKG_VERSION`。三处必须相同。

**Tech Stack:** 桌面 Vite 包、Tauri 2、Cargo。

## Global Constraints

- 不得声称 Mac App Store / Microsoft Store 已上架。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 对齐版本号

**Files:** `desktop/package.json` · `desktop/package-lock.json` · `desktop/src/components/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（`package.json` 与 Cargo / `tauri.conf.json` 同号；更新页展示该号；稳定通道 tag 与包版本相同时 `available` 为 false）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 把桌面包版本改成与 Tauri 构建相同**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
