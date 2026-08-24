# 使用向导逐步填写

> **给 Agent：** 不是新里程碑。只补 variables「逐步填写」的自动化映射。不要接广场。不要加语言步之外的云能力。

**Goal:** 主窗口使用向导对多个变量一次只问一个；有测试映射。

**Architecture:** 现有 `UsePromptModal.vue` 已按变量下标前进。本切片只加稳定测试钩子与规格映射。

**Tech Stack:** Vue Test Utils / Vitest。

## Global Constraints

- 本地优先；向导不把正文传到本机以外。
- 未填变量仍保留 `{{名称}}`。
- 双语关闭仍不删正文（已有映射，不得回退）。
- 没有本计划之外的应用文件。

---

### Task 1: 逐步填写测试与映射

**Files:**

- `desktop/src/components/UsePromptModal.vue`
- `desktop/src/components/UsePromptModal.spec.js`
- `docs/specs/variables/spec.md`

- [x] **Step 1: Write the failing test**
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 最小测试钩子；行为已存在则不改流程**
- [x] **Step 4: 测试 PASS；映射「逐步填写」**
- [x] **Step 5: `./scripts/docs-check`**
