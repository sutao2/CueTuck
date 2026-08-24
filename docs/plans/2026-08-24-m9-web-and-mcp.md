# M9 浏览器工作台与本机 MCP

> **给 Agent：** 先 MCP 后 Web。不要做原生移动端。不要做 ChatGPT 插件。不要假装云同步。启动器仍不请求广场。

**Goal:** Codex 等宿主能经 MCP 查本机提示词；浏览器有独立流体工作台。

**Architecture:** `mcp/` Rust stdio 读 `PROMPTARK_LIBRARY_DIR/promptark.sqlite`。`web/` 独立 Vue SPA，不进桌面包。

**Tech Stack:** rusqlite、MCP JSON-RPC stdio、Vue 3 + Vite。

## Global Constraints

- 本地优先。MCP 不联网查库。
- 浏览器库与桌面库未同步前不得写成已同步。
- 管理端仍独立，不进桌面包。

---

### Task 0: 接受合同（文档）

**Files:** `docs/changes/m9-web-and-mcp/` · ADR 0011 · specs/web · specs/mcp · 本计划 · INDEX

- [x] **Step 1: 提案、ADR、规格、里程碑、INDEX**
- [x] **Step 2: `./scripts/docs-check`**

### Task 1: MCP 搜索与读取

**Files:** `mcp/`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 最小 stdio 工具：search_prompts / get_prompt**
- [ ] **Step 4: 测试 PASS；映射「列出工具」「按标题命中」「缺库文件」「搜索不联网」**
- [ ] **Step 5: `./scripts/docs-check`**

### Task 2: MCP 渲染

**Files:** `mcp/`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: render_prompt，未填保留占位**
- [ ] **Step 4: 测试 PASS；映射「未填保留占位」**
- [ ] **Step 5: `./scripts/docs-check`**

### Task 3: 浏览器工作台壳

**Files:** `web/` · `desktop/src/platform/packageIsolation.test.js`

- [ ] **Step 1: Write the failing test**
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 独立 web 应用 + 窄桌面可收起侧栏；不声称已同步**
- [ ] **Step 4: 测试 PASS；映射 web 三场景**
- [ ] **Step 5: `./scripts/docs-check`**
