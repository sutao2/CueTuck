# 工作台壳层接线

> **给 Agent：** 不是新里程碑。内容区已画出的模型筛选、本地最近/收藏、右键菜单、界面语言、模型标签、变量建议、本机模型目录 MUST 接到真实行为。不得声称商店上架。启动器仍只搜本地。变量建议 MUST NOT 把正文送到本机以外。

**Goal:** 工作台里已经出现的控件按规格或本机设置生效，不再只改高亮。

**Architecture:** 广场 `model` 查询接到已有列表接口。本地最近按 `last_used_at`；本地收藏存本机设置 `local_favorite_ids`。右键只调用已有打开/编辑/使用/下载/收藏/删除。模型目录解析在 JS；语言包只覆盖壳层与设置文案。

**Tech Stack:** 桌面工作台、library / square 客户端、settings / square / workbench 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。
- 启动器 MUST NOT 请求广场。

---

### Task 1: 广场模型筛选

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `desktop/src-tauri/src/commands/square.rs` · `docs/specs/square/spec.md`

- [x] **Step 1: Write the failing test**（选模型后列表请求带 model；下拉含本机目录与条目上的模型名）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 下拉与 listSquareItems / Tauri query 接 model**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 本地最近、收藏与右键

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/workbench/spec.md`

- [x] **Step 1: Write the failing test**（最近只含已使用；收藏只含本机星标；右键能编辑/删除/使用已有动作）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: last_used_at、local_favorite_ids、context menu**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 3: 模型标签与本机模型目录

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（打开显示模型标签后卡片出现 model；新建用默认模型；目录进入下拉）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 卡片标签、编辑器模型、目录解析**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 4: 变量智能建议

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（打开后向导出现本机建议；关闭不出现；不发网络）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 本机词典，不读正文出站**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 5: 界面语言 English

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（顶栏 EN 或设置 English 后壳层文案切换；再选中文恢复）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 本机语言包覆盖壳层与设置导航**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 6: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
