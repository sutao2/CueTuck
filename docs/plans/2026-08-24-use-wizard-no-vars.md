# 使用向导无变量与 Enter

> **给 Agent：** 不是新里程碑。只补 variables「无变量」映射，并锁住已写明的 Enter / Shift+Enter。不要接广场。不要加语言步之外的云能力。

**Goal:** 无 `{{` 时向导直接预览；填写步 Enter 前进、Shift+Enter 换行。

**Architecture:** `UsePromptModal.vue` 已在无变量时进入 preview。本切片先映射该行为，再给填写框补键盘合同。

**Tech Stack:** Vue Test Utils / Vitest。

## Global Constraints

- 本地优先；向导不把正文传到本机以外。
- 未填变量仍保留 `{{名称}}`。
- 双语关闭仍不删正文（已有映射，不得回退）。
- 没有本计划之外的应用文件。不发明 M9。

---

### Task 1: 无变量直接预览

**Files:**

- `desktop/src/components/UsePromptModal.spec.js`
- `docs/specs/variables/spec.md`

- [x] **Step 1: Write the failing test**
- [x] **Step 2: Run test — FAIL 或确认现有钩子已足够**
- [x] **Step 3: 行为已存在则不改流程**
- [x] **Step 4: 测试 PASS；映射「无变量」**
- [x] **Step 5: `./scripts/docs-check`**

### Task 2: Enter 前进、Shift+Enter 换行

**Files:**

- `desktop/src/components/UsePromptModal.spec.js`
- `desktop/src/components/UsePromptModal.vue`
- `docs/specs/variables/spec.md`

- [x] **Step 1: Write the failing test**
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 只在填写框处理 Enter / Shift+Enter**
- [x] **Step 4: 测试 PASS；映射该场景**
- [x] **Step 5: `./scripts/docs-check`**
