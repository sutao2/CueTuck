# 浏览器使用向导

> **给 Agent：** 不是新里程碑。浏览器内存库使用流程对齐 variables：一次一个变量，未填保留 `{{名称}}`。不要把正文传到本机以外。不要写桌面 SQLite。不要从 `desktop/` import（可抄同一替换规则）。

**Goal:** 浏览器打开带 `{{变量}}` 的提示词后能逐步填写、预览、复制；无变量则直接预览。

**Architecture:** 在 `web/` 内实现与 `desktop/src/lib/renderPrompt.js` 相同的解析与渲染规则。向导只读当前标签页内存库。

**Tech Stack:** Vue 3、Vitest、`navigator.clipboard`（测试里 mock）。

## Global Constraints

- 未填保留 `{{名称}}`。
- 不声称与桌面库已同步。
- 不进桌面包。启动器仍只搜本地。
- 没有本计划之外的应用文件。

---

### Task 0: 规格场景

**Files:** `docs/specs/web/spec.md` · 本计划

- [ ] **Step 1: 增加浏览器向导场景：逐步填写、无变量直接预览、漏填保留占位；映射先写「未开始」。不把 desktop 主窗口向导场景改到 web**
- [ ] **Step 2: `./scripts/docs-check`**

### Task 1: 解析与渲染

**Files:** `web/src/renderPrompt.js` · `web/src/renderPrompt.spec.js`

- [ ] **Step 1: Write the failing test**（重复 `{{产品}}` 只一个名；未填保留 `{{受众}}`）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: `extractVariables` / `renderPrompt`，规则与桌面相同**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check`**

### Task 2: 向导 UI

**Files:** `web/src/WebApp.vue` · `web/src/WebApp.spec.js`

- [ ] **Step 1: Write the failing test**（正文 `{{城市}}` 与 `{{天数}}`：先只填城市，下一步天数，预览为替换结果；点复制时调用 clipboard）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 「使用」打开向导；每次一个变量；Enter 前进（测试可点下一步）。无 `{{` 跳过填写进预览**
- [ ] **Step 4: 测试 PASS；映射 web 向导三场景**
- [ ] **Step 5: `./scripts/docs-check`**
