# M5 Online Square Implementation Plan

> **For agentic workers:** 合同已接受（[ADR 0008](../architecture/decisions/0008-m5-backend-contract.md)）。Task 0–1 已完成。Task 2 起才创建 `backend/`。不要接管理台。不要写死 QQ / LinuxDo。启动器不请求广场。

**Goal:** 联网后浏览广场、匿名下载、登录后收藏与发布；断网时本地与启动器仍全部可用。

**Architecture:** 新 OpenAPI，不接旧 Spring 端点。Rust 发请求并持有 Refresh。下载落入本机 SQLite。启动器仍只搜本地库。

**Tech Stack:** Vue 3、Vitest、Tauri 2、rusqlite；`backend/` 为 Rust + Axum。Refresh 用 `keyring`。无 Docker 时 Testcontainers 保持 ignore。

## Global Constraints

- 本地功能必须离线可用。云端不是使用前提。
- 启动器必须是独立窗口，label 保持 `launcher`。
- Refresh token 只进系统钥匙串。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`。

---

### Task 0: 接受合同并写进现行文档

**Files:**

- Create: 下一号 ADR（改写 API、邮箱密码、覆盖率）
- Modify: `docs/architecture/overview.md`（补后端容器，仍不画旧 Spring 路径）
- Modify: `docs/reference/test-gates.md`（抄提案里的 80% / 70%）
- Modify: `docs/changes/m5-backend-contract/proposal.md`（状态改为接受）
- Modify: `docs/INDEX.md`

- [x] **Step 1:** 提案状态变为「接受」（人确认，Agent 不得自称接受）
- [x] **Step 2:** 写 ADR；门禁写入覆盖率；INDEX 更新
- [x] **Step 3:** `./scripts/docs-check`

未完成 Task 0，不得开始 Task 1。

---

### Task 1: OpenAPI 与映射测试

**Files:**

- Create: `docs/reference/openapi/square.yaml`
- Test: 客户端映射测试（每个 path 一条，先对契约、不对真服务）

- [x] **Step 1: Write the failing tests**
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小合同：浏览、下载、登录、收藏、发布**
- [x] **Step 4: 测试 PASS；INDEX 登记 OpenAPI**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 2: 邮箱密码与钥匙串

**Files:**

- `backend/` 认证
- `desktop/src-tauri` 登录命令与钥匙串
- `desktop/src` 登录弹窗（写明触发原因）

- [x] **Step 1: Write the failing tests**
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小实现；浏览器预览不持久化 Refresh**
- [x] **Step 4: 测试 PASS；映射 auth「发布触发」「令牌」**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 3: 广场浏览与离线

**Files:**

- 广场列表 / 筛选
- 工作台广场空间（替换「未开放」）

- [x] **Step 1: Write the failing tests**（离线说明 + 回本地；无网时本地仍可用）
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小实现；启动器不增加广场请求**
- [x] **Step 4: 测试 PASS；映射 square「离线」**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 4: 匿名下载与收藏分离

- [ ] **Step 1: Write the failing tests**（未登录下载写入 `source=downloaded`；未登录收藏只开登录、不写库）
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现**
- [ ] **Step 4: 测试 PASS；映射 square「未登录下载」「未登录收藏」**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 5: 发布不锁本地

- [ ] **Step 1: Write the failing tests**（未选源禁用提交；提交后本地仍可编辑）
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现**
- [ ] **Step 4: 测试 PASS；映射 publish「未选源」「审核与本地并行」**
- [ ] **Step 5: `./scripts/docs-check`**
