# 自动更新安装

> **给 Agent：** 不是新里程碑。检查更新必须是真请求。没有发行物就说没有可用更新。不得声称已上架商店。启动器不负责安装更新。

**Goal:** 设置更新页能检查 GitHub Releases、展示发行说明、按通道下载并安装；无发行物时不假装已从商店安装。

**Architecture:** Tauri updater 指向本仓库 Releases。自动下载是本机开关。

**Tech Stack:** tauri-plugin-updater、GitHub Releases。

## Global Constraints

- 不得声称 Mac App Store / Microsoft Store 已上架。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 检查更新

**Files:** `desktop/src/components/SettingsModal.vue` · `desktop/src-tauri/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（点检查更新后无发行物则说明没有可用更新；不出现已从商店）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 真请求 GitHub Releases；无发行物诚实说明**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 按通道下载安装

**Files:** `desktop/src/` · `desktop/src-tauri/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（自动下载打开且通道有包时能排队安装；关闭本计划）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 本机开关控制自动下载；安装走 updater 而不是商店**
- [ ] **Step 4: 测试 PASS；关闭本计划 done 记录**
- [ ] **Step 5: `./scripts/docs-check` 并提交**
