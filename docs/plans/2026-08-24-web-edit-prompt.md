# 浏览器工作台编辑

> **给 Agent：** 不是新里程碑。只补 `web/` 内存库改标题与正文。不要写桌面 SQLite。不要声称已同步。不要接广场。

**Goal:** 浏览器本地空间能打开已有提示词并保存修改，列表立即反映新标题。

**Architecture:** 沿用 `web/src/memoryLibrary.js`。编辑态复用新建表单。刷新标签页仍丢失。

**Tech Stack:** Vue 3、Vitest。

## Global Constraints

- 浏览器库与桌面库未同步前不得写成已同步。
- 不进桌面包。不从 `desktop/` import。
- 启动器与 MCP 不请求广场。
- 没有本计划之外的应用文件。

---

### Task 0: 规格场景

**Files:** `docs/specs/web/spec.md` · 本计划

- [x] **Step 1: 增加「编辑后列表更新」场景；映射先写「未开始」**
- [x] **Step 2: `./scripts/docs-check`**

### Task 1: 保存编辑

**Files:** `web/src/memoryLibrary.js` · `web/src/WebApp.vue` · `web/src/WebApp.spec.js`

- [x] **Step 1: Write the failing test**（新建「测试」后打开，改标题为「已改」并保存，列表出现「已改」且不含旧标题；说明仍写尚未与桌面同步）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: `updateLocalPrompt({ id, title, content })`；详情可进入编辑；空标题不保存**
- [x] **Step 4: 测试 PASS；映射「编辑后列表更新」**
- [x] **Step 5: `./scripts/docs-check`**
