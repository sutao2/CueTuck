# M6 Admin Console Implementation Plan

> **For agentic workers:** 合同已接受（[ADR 0009](../architecture/decisions/0009-m6-admin-console.md)）。Task 0–4 已完成。不要接 OAuth。不要把管理页打进桌面。启动器不请求管理接口。

**Goal:** 独立浏览器管理端能审核待发、只读看用户、改一项运行时开关；桌面安装包不含管理代码。

**Architecture:** 新 OpenAPI `admin.yaml`，同一 Axum `backend/`，新前缀。`admin-web/` 只持内存 Access。审核只改远端状态，不碰作者 SQLite。

**Tech Stack:** Vue 3、Vitest、Vite；`backend/` Rust + Axum。无 Docker 时 Testcontainers 保持 ignore。

## Global Constraints

- 本地功能必须离线可用。云端不是使用前提。
- 启动器必须是独立窗口，label 保持 `launcher`。
- 管理端 Refresh 不进 Web Storage。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`。
- 不把本机 `backend/` 写成生产。

---

### Task 0: 接受合同并写进现行文档

**Files:**

- Create: 下一号 ADR（独立管理合同、预发=`backend/`、`admin-web` 不进桌面）
- Modify: `docs/changes/m6-admin-console/proposal.md`（状态改为接受）
- Modify: `docs/architecture/overview.md`（补管理端容器，不画进桌面包）
- Modify: `docs/INDEX.md`

- [x] **Step 1:** 提案状态变为「接受」（人确认，Agent 不得自称接受）
- [x] **Step 2:** 写 ADR；INDEX 更新
- [x] **Step 3:** `./scripts/docs-check`

未完成 Task 0，不得开始 Task 1。

---

### Task 1: 管理 OpenAPI 与映射测试

**Files:**

- Create: `docs/reference/openapi/admin.yaml`
- Test: 每个 path 一条客户端映射测试（先对契约、不对真服务）

- [x] **Step 1: Write the failing tests**
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小合同：登录角色、待审列表、通过/驳回、用户列表、一项设置**
- [x] **Step 4: 测试 PASS；INDEX 登记 OpenAPI**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 2: 管理员角色与审核 API

**Files:**

- `backend/` 角色、`/v1/admin/*`、普通令牌拒绝

- [x] **Step 1: Write the failing tests**（普通令牌不能审核；通过/驳回改远端状态）
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小实现；不写桌面库**
- [x] **Step 4: 测试 PASS；映射 admin「普通令牌不能审核」「列出待审」**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 3: 独立 admin-web 登录与审核页

**Files:**

- Create: `admin-web/`（不得加入桌面打包）
- Test: 管理端不把 Refresh 写入 Web Storage；审核列表可操作

- [x] **Step 1: Write the failing tests**
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小实现**
- [x] **Step 4: 测试 PASS；映射「管理端不持久化 Refresh」「列出待审」**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 4: 用户只读列表

**Files:**

- `backend/` 用户列表
- `admin-web/` 用户页（无改密、无删除）

- [x] **Step 1: Write the failing tests**
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小实现**
- [x] **Step 4: 测试 PASS；映射「看到邮箱与角色」**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 5: 运行时开关与桌面包隔离

**Files:**

- `backend/` 一项匿名广场开关
- `admin-web/` 设置页
- `desktop/` 断言包内无 `admin-web`；启动器仍无管理请求

- [ ] **Step 1: Write the failing tests**（关开关后匿名列表失败或空；桌面依赖不含 admin-web；启动器不请求管理）
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现**
- [ ] **Step 4: 测试 PASS；映射「关闭公开广场」「桌面包不含管理」「启动器不请求管理」**
- [ ] **Step 5: `./scripts/docs-check`**
