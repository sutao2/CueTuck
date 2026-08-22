# M1 Desktop Skeleton Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `desktop/` 落地可运行的 Vue 3 + Tauri 2 骨架：空工作台壳、SQLite 可初始化、启动器窗口可唤起。

**Architecture:** 应用只放在 `desktop/`，给后期 `backend/` 留根目录。主窗口与启动器是两个 Webview。本地库经 Rust 命令初始化，前端只问状态。第一期不发任何产品 API。

**Tech Stack:** Vue 3、Vite、Tauri 2、SQLite（`tauri-plugin-sql` 或 Rust 侧 rusqlite，实现时选一个并写 ADR）、前端测试用 Vitest、Rust 用 `cargo test`。

## Global Constraints

- 本地功能离线可用，云端不是使用前提。
- 启动器必须是独立窗口，不得改成主窗口覆盖层。
- 第一期不接后端，不实现 OAuth / 广场请求。
- 主窗口视觉跟「提示词软件 2」，不沿用 Atelier Zero 当主主题。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`（在仓库根）。

---

### Task 1: 建立 desktop 包装与探活测试

**Files:**

- Create: `desktop/package.json`
- Create: `desktop/vite.config.js`
- Create: `desktop/index.html`
- Create: `desktop/src/main.js`
- Create: `desktop/src/App.vue`
- Create: `desktop/src/app.test.js`
- Modify: `docs/how-to/local-dev.md`（写入真实 `cd desktop && npm test`）
- Modify: `docs/INDEX.md`（若新增 Markdown）

**Interfaces:**

- Consumes: 无
- Produces: `desktop` 包；`npm test` 在 `desktop/` 可运行

- [ ] **Step 1: Write the failing test**

```js
import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = dirname(fileURLToPath(import.meta.url));

describe("desktop package", () => {
  it("declares a vue app entry", () => {
    const pkg = JSON.parse(readFileSync(join(root, "..", "package.json"), "utf8"));
    expect(pkg.name).toBe("promptark-desktop");
    expect(pkg.scripts.test).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd desktop && npm test`

Expected: FAIL，因为还没有 `package.json` 或脚本。

- [ ] **Step 3: Write minimal implementation**

初始化 Vite + Vue，包名 `promptark-desktop`，`scripts.test` 指向 Vitest。`App.vue` 先渲染一个带 `data-testid="app-shell"` 的空节点即可。

- [ ] **Step 4: Run test to verify it passes**

Run: `cd desktop && npm test`

Expected: PASS

- [ ] **Step 5: Update spec test mapping and INDEX if needed**

- [ ] **Step 6: Run `./scripts/docs-check`**

---

### Task 2: SQLite 初始化命令

**Files:**

- Create: `desktop/src-tauri/src/commands/database.rs`
- Create: `desktop/src-tauri/src/local_database/`（状态机：Pending / Ready / Failed）
- Create: `desktop/src-tauri/src/local_database/tests.rs`
- Modify: `desktop/src-tauri/src/lib.rs`（注册命令）
- Modify: `docs/architecture/data-model.md`（记下实际采用的访问方式）
- Create: `docs/architecture/decisions/0007-sqlite-access.md`（sql plugin 或 rusqlite，二选一）

**Interfaces:**

- Consumes: 无
- Produces: `initialize_local_database`、`get_local_database_status`；状态值为 `pending` | `ready` | `failed`

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn status_is_ready_after_initialize() {
    let dir = tempfile::tempdir().unwrap();
    let status = initialize_in_dir(dir.path()).await.unwrap();
    assert_eq!(status, "ready");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd desktop/src-tauri && cargo test --locked status_is_ready_after_initialize`

Expected: FAIL，符号不存在。

- [ ] **Step 3: Write minimal implementation**

在临时目录创建 SQLite 文件，写入空库（可暂无业务表，或只建 `settings`）。不要在这一任务实现 prompts CRUD。

- [ ] **Step 4: Run test to verify it passes**

Run: `cd desktop/src-tauri && cargo test --locked`

Expected: PASS

- [ ] **Step 5: Update spec test mapping**

在 [workbench spec](../specs/workbench/spec.md)「SQLite 就绪」映射到该测试名。

- [ ] **Step 6: Run `./scripts/docs-check`**

---

### Task 3: 主窗口四段壳

**Files:**

- Create: `desktop/src/styles/tokens.css`（从旧仓库原型 `styles.css` 抽 token，不整文件抄交互 JS）
- Create: `desktop/src/components/WorkbenchShell.vue`
- Create: `desktop/src/components/WorkbenchShell.spec.js`
- Modify: `desktop/src/App.vue`

**Interfaces:**

- Consumes: 无
- Produces: 可测试的 `titlebar` / `sidebar` / `content` / `statusbar` 区域

- [ ] **Step 1: Write the failing test**

```js
import { mount } from "@vue/test-utils";
import { describe, it, expect } from "vitest";
import WorkbenchShell from "./WorkbenchShell.vue";

describe("WorkbenchShell", () => {
  it("renders four chrome regions", () => {
    const w = mount(WorkbenchShell);
    expect(w.get('[data-region="titlebar"]').exists()).toBe(true);
    expect(w.get('[data-region="sidebar"]').exists()).toBe(true);
    expect(w.get('[data-region="content"]').exists()).toBe(true);
    expect(w.get('[data-region="statusbar"]').exists()).toBe(true);
  });

  it("keeps square offline on first phase", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    expect(w.get('[data-testid="square-unavailable"]').exists()).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd desktop && npm test -- WorkbenchShell`

Expected: FAIL，组件不存在。

- [ ] **Step 3: Write minimal implementation**

按原型结构排四段。本地空间为默认。广场点击只显示未开放文案，不 `fetch`。

- [ ] **Step 4: Run test to verify it passes**

Expected: PASS

- [ ] **Step 5: 映射 workbench「打开应用」「第一期点击广场」**

- [ ] **Step 6: Run `./scripts/docs-check`**

---

### Task 4: 启动器窗口显示与隐藏

**Files:**

- Create: `desktop/launcher.html`
- Create: `desktop/src/launcher.js`
- Create: `desktop/src/LauncherApp.vue`
- Create: `desktop/src-tauri/src/commands/launcher.rs`
- Create: `desktop/src/platform/launcherWindow.test.js`
- Modify: Tauri 窗口与全局快捷键配置

**Interfaces:**

- Consumes: `get_local_database_status`
- Produces: `show_launcher`、`hide_launcher`、`toggle_launcher`；窗口 label 必须是 `launcher`

- [ ] **Step 1: Write the failing test**

```js
import { describe, it, expect } from "vitest";
import { LAUNCHER_LABEL } from "./launcherWindow.js";

describe("launcher window", () => {
  it("uses the independent window label", () => {
    expect(LAUNCHER_LABEL).toBe("launcher");
  });
});
```

Rust 侧增加：

```rust
#[test]
fn launcher_label_is_stable() {
    assert_eq!(crate::commands::launcher::LAUNCHER_LABEL, "launcher");
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Write minimal implementation**

独立窗口。快捷键先用可配置默认（与旧产品接近即可）。空库时启动器显示「正在准备本地数据」或空态，不请求网络。不要在本任务移植填写/粘贴。

- [ ] **Step 4: Run tests to verify they pass**

- [ ] **Step 5: 映射 launcher「独立窗口」「空查询」中与 M1 重叠的部分；其余保持未开始**

- [ ] **Step 6: Run `./scripts/docs-check`**

---

### Task 5: 收口文档与状态

**Files:**

- Modify: `docs/plans/status.md`
- Modify: `docs/plans/milestones/m1.md` 退出标准
- Create: `docs/plans/done/YYYY-MM-DD-m1-desktop-skeleton.md`（仅当上述任务全绿）
- Modify: `docs/how-to/local-dev.md`
- Modify: `docs/specs/workbench/spec.md` 测试映射
- Modify: `docs/specs/launcher/spec.md` 测试映射（仅 M1 已做场景）

**Interfaces:**

- Consumes: Task 1–4 的真实命令
- Produces: 完成记录；status 中 M1 为完成

- [ ] **Step 1: 确认没有应用代码跑红**

Run: `cd desktop && npm test` 与 `cd desktop/src-tauri && cargo test --locked`

- [ ] **Step 2: 填写完成记录模板，粘贴命令输出摘要**

- [ ] **Step 3: 更新 status.md，勾选 m1.md 退出标准**

- [ ] **Step 4: Run `./scripts/docs-check`**

- [ ] **Step 5: 不要开始 M2 文件树以外的功能**
