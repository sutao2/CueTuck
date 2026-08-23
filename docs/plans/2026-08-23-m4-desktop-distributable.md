# M4 Desktop Distributable Implementation Plan

> **For agentic workers:** 按任务顺序执行。步骤使用 `- [ ]`。未完成计划不得开始 M5，不得商店上架，不得在签名材料未齐时声称可公开下载。不要整仓库复制旧前端。

**Goal:** 让本机开发/调试可重复，补上 SQLite 库文件备份与恢复，并把文档、规格、宣传口径与实现对齐。

**Architecture:** JSON 导出/导入已存在，不算备份。备份是复制 `promptark.sqlite`；恢复先校验再替换，失败回滚，不写半份库。本机 smoke 用手工 QA 表，不把未验证平台写成已支持。Playwright CI 本里程碑不启用。

**Tech Stack:** 既有 Vue 3、Vitest、Tauri 2、rusqlite；不新增商店/签名依赖。

## Global Constraints

- 备份与恢复 MUST 可取消失败且不写半份数据（[settings](../specs/settings/spec.md)）。
- 业务只走 Rust 命令（ADR 0007）。
- 启动器必须是独立窗口，label 保持 `launcher`。
- 第一期不接广场、登录、发布。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`。
- 只宣传本机已验证的平台；当前开发机是 macOS。

---

### Task 1: SQLite 库文件备份与恢复

**Files:**
- Create: `desktop/src-tauri/src/local_database/backup.rs`
- Modify: `desktop/src-tauri/src/local_database/mod.rs`
- Modify: `desktop/src-tauri/src/local_database/tests.rs`
- Modify: `desktop/src-tauri/src/commands/database.rs`
- Modify: `desktop/src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

在 `tests.rs` 增加：

```rust
#[tokio::test]
async fn restore_replaces_library() { /* A → backup → B → restore → 只剩 A */ }

#[tokio::test]
async fn failed_restore_leaves_library() { /* 恢复垃圾文件 → 仍是 A */ }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked restore_replaces_library failed_restore_leaves_library`
Expected: compile error（函数不存在）或 FAIL。

- [ ] **Step 3: Write minimal implementation**

`backup_library_in_dir(dir, dest)` 复制 `promptark.sqlite`。
`restore_library_in_dir(dir, src)` 先打开 src 确认有 `prompts` 表，再替换 live 文件；拷贝失败或校验失败则从 `.restore-bak` 回滚。
Tauri：`backup_local_library` / `restore_local_library`。`dest` 为空时写到 `app_data_dir/backups/promptark-<utc>.sqlite`。

- [ ] **Step 4: Run tests**

Run: `cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked`
Expected: PASS（忽略的 1 万条 bench 除外）。

- [ ] **Step 5: Docs-check**

Run: `./scripts/docs-check`

---

### Task 2: 设置页接上备份恢复

**Files:**
- Modify: `desktop/src/platform/library.js`
- Modify: `desktop/src/platform/library.test.js`
- Modify: `desktop/src/components/SettingsModal.vue`
- Modify: `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing tests**

`library.test.js`：浏览器内存模式调用库文件备份/恢复时抛「仅桌面窗口支持库文件备份」，且提示词条数不变。

- [ ] **Step 2: Run to verify they fail**

Run: `cd desktop && npm test -- src/platform/library.test.js`
Expected: FAIL。

- [ ] **Step 3: Write minimal implementation**

`backupLocalLibrary` / `restoreLocalLibrary`：Tauri 走命令；浏览器抛上述错误。
设置「数据与备份」增加备份、恢复路径与按钮；失败文案可见。恢复成功后 `emit('imported')` 刷新工作台。
规格补「备份恢复」场景，测试映射指向 Rust 测试名与 `library.test.js`。

- [ ] **Step 4: Run tests**

Run: `cd desktop && npm test`
Expected: PASS。

- [ ] **Step 5: Docs-check**

Run: `./scripts/docs-check`

---

### Task 3: 本机文档与宣传口径

**Files:**
- Modify: `docs/how-to/local-dev.md`
- Modify: `README.md`
- Create: `docs/how-to/release-qa.md`
- Modify: `docs/INDEX.md`
- Modify: `docs/reference/test-gates.md`

- [ ] **Step 1: Rewrite local-dev**

去掉「应用尚未开始 / M2 正在」过时句。写清：浏览器预览、`tauri dev`、备份恢复、启动器快捷键。写明已验证平台是 macOS；Windows/Linux 不得写成已支持。

- [ ] **Step 2: Add release QA checklist**

`docs/how-to/release-qa.md`：新建提示词、分类筛选、启动器搜索/Esc、设置快捷键冲突、JSON 导入预览、库文件备份恢复、广场无网络。首次执行时勾选并记下日期与机器。不声称商店包。

- [ ] **Step 3: Align gates**

`test-gates` M4：本机 smoke = 手工 QA 表 + 备份单测。Playwright CI 标「未启用」，不得在 README 写成已通过。

- [ ] **Step 4: Docs-check**

Run: `./scripts/docs-check`

---

### Task 4: 规格映射审计与关闭

**Files:**
- Modify: `docs/specs/*/spec.md`（仅补「已实现却未映射」的测试列）
- Modify: `docs/plans/milestones/m4.md`
- Create: `docs/plans/done/2026-08-23-m4-desktop-distributable.md`
- Modify: `docs/INDEX.md`
- Modify: `docs/plans/status.md`
- Modify: `docs/product/roadmap.md`

- [ ] **Step 1: Audit MUST mappings**

打开已实现能力规格的测试映射表。缺行就补；未实现 MUST 保持未映射并写在完成记录「未做」。

- [ ] **Step 2: Fill QA once**

按 `how-to/release-qa.md` 走一遍（浏览器 + 能跑则 `tauri dev`）。勾选写入完成记录。

- [ ] **Step 3: Close milestone**

归档本计划、写 done、更新 status/roadmap/m4。不打可公开下载标签，不开始 M5。

- [ ] **Step 4: Docs-check**

Run: `./scripts/docs-check`
