# Windows / Linux 开机启动与托盘

> **给 Agent：** 不是新里程碑。写出开机启动与关闭后托盘的行为。未在该 OS 手工验证前，`release-qa` 不得勾选。不得声称 Windows NSIS 已验证。

**Goal:** Windows 与 Linux 上开机启动、最小化到托盘与 macOS 已有行为对齐；保存失败时开关回退。

**Architecture:** 现有桌面偏好命令按目标 OS 分支实现。

**Tech Stack:** 现有 Tauri 桌面命令。

## Global Constraints

- 未验证平台不得勾选发行 QA。
- GitHub 额度恢复前不重跑 Windows NSIS 工作流，除非人明确要求。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: Windows 开机启动与托盘

**Files:** `desktop/src-tauri/src/commands/desktop.rs` · `desktop/src/platform/desktopPrefs.js` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（Windows 上打开开机启动与托盘能保存；失败时开关回退；不声称 NSIS 已验证）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 按 Windows 实现现有偏好命令，保存失败回退**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: Linux 开机启动与托盘

**Files:** `desktop/src-tauri/src/commands/desktop.rs` · `desktop/src/platform/desktopPrefs.js` · `docs/specs/settings/spec.md` · `docs/how-to/release-qa.md`

- [ ] **Step 1: Write the failing test**（Linux 上打开开机启动与托盘能保存；失败时开关回退；未手工验证不勾发行 QA）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 按 Linux 实现现有偏好命令；关闭本计划 done 记录**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**
