# M2 Local Workbench Implementation Plan

> **For agentic workers:** 按任务顺序执行。步骤使用 `- [ ]`。未完成计划不得开始 M3。

**Goal:** 在已有工作台壳上落地本机提示词 CRUD、两级分类、合集、使用向导和本机设置。

**Architecture:** 业务只走 Rust 命令（ADR 0007）。Vue 负责壳、弹窗和列表。分类预置写入 SQLite，侧栏不再写死树。变量只从正文解析，不建变量表。

**Tech Stack:** Vue 3、Vitest、Tauri 2、rusqlite、既有 `initialize_in_dir`。

## Global Constraints

- 本地功能离线可用，云端不是使用前提。
- 启动器必须是独立窗口，不得改成主窗口覆盖层。
- 第一期不接后端，广场不发网络请求。
- 主窗口视觉跟「提示词软件 2」。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`。

---

### Task 1: 补齐 schema 并预置分类

**Files:**

- Modify: `desktop/src-tauri/src/local_database/mod.rs`
- Modify: `desktop/src-tauri/src/local_database/tests.rs`
- Modify: `docs/architecture/data-model.md`（记下实际列）
- Modify: `docs/specs/categories/spec.md` 测试映射

**Interfaces:**

- Consumes: `initialize_in_dir`
- Produces: `list_categories_in_dir(dir) -> Vec<CategoryRecord>`；十大系统大分类及首包小分类；`prompts` 含 `category_id` / `collection_id` / `model` / `source` / `use_count` / `last_used_at` / `created_at` / `updated_at` / `deleted_at`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn seeds_ten_system_categories() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let names = list_system_category_names(dir.path()).unwrap();
    assert_eq!(names, vec![
        "软件开发", "图片生成", "视频创作", "办公效率", "内容写作",
        "产品设计", "市场营销", "数据分析", "教育学习", "生活助手",
    ]);
}
```

- [ ] **Step 2: Run `cargo test --locked seeds_ten_system_categories`** — FAIL
- [ ] **Step 3: 建 `categories` 表，幂等插入预置分类；扩展 `prompts` 列**
- [ ] **Step 4: `cargo test --locked` PASS**
- [ ] **Step 5: 映射 categories「空库首次打开」**
- [ ] **Step 6: `./scripts/docs-check`**

---

### Task 2: 新建与列表

**Files:**

- Create: `desktop/src-tauri/src/local_database/prompts.rs`
- Modify: `desktop/src-tauri/src/commands/database.rs`
- Create: `desktop/src/platform/library.js`
- Modify: `desktop/src/components/WorkbenchShell.vue`
- Create: `desktop/src/components/CreatePromptModal.vue`
- Test: `desktop/src-tauri` `creates_prompt_and_lists_it`；`desktop/src/platform/library.test.js`

**Interfaces:**

- Consumes: `initialize_in_dir`
- Produces: `create_local_prompt({ title, content, category_id }) -> PromptRecord`；`list_local_prompts({ query, category_id }) -> Vec<PromptRecord>`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn creates_prompt_and_lists_it() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    create_prompt_in_dir(dir.path(), "测试", "正文", None).unwrap();
    let rows = list_prompts_in_dir(dir.path(), "测试", None).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "测试");
}
```

- [ ] **Step 2: FAIL**
- [ ] **Step 3: 实现命令；工作台「新建提示词」弹出创建框，保存后卡片出现**
- [ ] **Step 4: PASS**
- [ ] **Step 5: 映射 library「新建并保存」**
- [ ] **Step 6: `./scripts/docs-check`**

---

### Task 3: 编辑、软删除、搜索

**Files:**

- Modify: `prompts.rs`、`WorkbenchShell.vue`、详情弹窗
- Test: `soft_deleted_prompt_is_hidden`；`search_hits_content`

**Interfaces:**

- Produces: `update_local_prompt`；`delete_local_prompt`（写 `deleted_at`）

- [ ] **Step 1:**

```rust
#[tokio::test]
async fn soft_deleted_prompt_is_hidden() {
    let dir = tempfile::tempdir().unwrap();
    initialize_in_dir(dir.path()).unwrap();
    let id = create_prompt_in_dir(dir.path(), "过期模板", "x", None).unwrap().id;
    delete_prompt_in_dir(dir.path(), &id).unwrap();
    assert!(list_prompts_in_dir(dir.path(), "过期", None).unwrap().is_empty());
}
```

- [ ] **Step 2–4:** 实现编辑弹窗与内容区搜索
- [ ] **Step 5:** 映射 library「删除后搜索」「按正文命中」
- [ ] **Step 6:** `./scripts/docs-check`

---

### Task 4: 侧栏树改读库

**Files:**

- Modify: `WorkbenchShell.vue`；`list_categories` 命令
- Test: 前端或 Rust `lists_children_under_software`

**Interfaces:**

- Produces: 树节点来自 SQLite；选中分类只筛选列表；合集不进树

- [ ] **Step 1:** 测「软件开发」下有网站开发等首包小分类
- [ ] **Step 3:** 去掉 Vue 内写死的 `PRESET_CATEGORIES`
- [ ] **Step 5:** 映射 categories「展开大分类」/ workbench 同名场景
- [ ] **Step 6:** `./scripts/docs-check`

---

### Task 5: 合集

**Files:**

- Modify: schema `collections`；`create_local_collection`；内容区混排
- Test: `creates_empty_collection`；`adds_member_via_collection_id`

**Interfaces:**

- Produces: 合集出现在内容区；`prompts.collection_id` 归属

- [ ] **Step 5:** 映射 collections「新建合集」「向合集添加」
- [ ] **Step 6:** `./scripts/docs-check`

---

### Task 6: 使用向导与计数

**Files:**

- Create: `desktop/src/lib/renderPrompt.js` + `.test.js`
- Create: 使用弹窗
- Modify: Rust `record_prompt_use`

**Interfaces:**

- Produces: `extractVariables(content)`；`renderPrompt(content, values)` 未填保留 `{{名称}}`；复制成功后 `use_count + 1`

- [ ] **Step 1:**

```js
it("dedupes repeated variables", () => {
  expect(extractVariables("为 {{产品}} 写介绍，再次强调 {{产品}}")).toEqual(["产品"]);
});
```

- [ ] **Step 5:** 映射 variables「重复变量」「逐步填写」；library「复制后计数」
- [ ] **Step 6:** `./scripts/docs-check`

---

### Task 7: 本机设置

**Files:**

- Create: 设置弹窗（常规 / 快捷键占位 / 数据 / 外观）
- Modify: `settings` 表读写主题
- Test: 主题写入后再次读取为 `dark`

**Interfaces:**

- Produces: 侧栏「设置」打开弹窗；外观持久化；同步页说明未提供；导入先预览（可先做导出 JSON + 预览结构）

- [ ] **Step 5:** 映射 settings「打开设置」「未实现页」「外观」
- [ ] **Step 6:** `./scripts/docs-check`

---

### Task 8: 收口

**Files:**

- Modify: `docs/plans/status.md`、`milestones/m2.md`
- Create: `docs/plans/done/YYYY-MM-DD-m2-local-workbench.md`（仅全绿）

- [x] **Step 1:** `npm test` 与 `cargo test --locked` 全绿
- [x] **Step 2:** 广场仍无 fetch
- [x] **Step 3:** 完成记录 + status
- [x] **Step 4:** `./scripts/docs-check`
- [x] **Step 5:** 不要开始 M3 整份移植
