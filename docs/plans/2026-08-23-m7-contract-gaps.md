# M7 Contract Gaps Implementation Plan

> **For agentic workers:** 提案仍是草稿。未完成 Task 0（人接受提案）不得实现收藏写路径、令牌轮换或 `GET /v1/admin/me`。不要接 OAuth。不要把管理页打进桌面。启动器不请求广场或管理接口。

**Goal:** 补齐已接受合同：已登录收藏、令牌轮换、管理员身份读取。

**Architecture:** 同一 Axum `backend/`。收藏是账号关系，不写作者本地库。Refresh 轮换后旧对失效；桌面新 Refresh 进钥匙串。

**Tech Stack:** 现有 Vue / Vitest / Axum。无 Docker 时 Testcontainers 保持 ignore。

## Global Constraints

- 本地功能必须离线可用。云端不是使用前提。
- 启动器必须是独立窗口，label 保持 `launcher`。
- Refresh 不进 Web Storage。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`。
- 不把本机 `backend/` 写成生产。

---

### Task 0: 接受合同并写进现行文档

**Files:**

- Create: 下一号 ADR（收藏写路径、轮换、`/v1/admin/me`）
- Modify: `docs/changes/m7-contract-gaps/proposal.md`（状态改为接受）
- Modify: `docs/INDEX.md`

- [ ] **Step 1:** 提案状态变为「接受」（人确认，Agent 不得自称接受）
- [ ] **Step 2:** 写 ADR；INDEX 更新
- [ ] **Step 3:** `./scripts/docs-check`

未完成 Task 0，不得开始 Task 1。

---

### Task 1: GET /v1/admin/me

**Files:**

- `backend/` 管理员身份
- 映射测试

- [ ] **Step 1: Write the failing tests**（管理员 200 + 角色；普通令牌拒绝）
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现**
- [ ] **Step 4: 测试 PASS；映射 admin「查询管理员身份」**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 2: 已登录收藏与取消收藏

**Files:**

- `backend/` `PUT`/`DELETE`/`GET` `/v1/favorites`
- `desktop/` 已登录收藏不写 downloaded 副本

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；启动器仍不请求广场**
- [ ] **Step 4: 测试 PASS；映射 square「已登录收藏」「取消收藏」**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 3: Access / Refresh 轮换

**Files:**

- OpenAPI 刷新 path（若尚无）
- `backend/` 轮换并使旧令牌失效
- `desktop/` 钥匙串只持新 Refresh

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；Web Storage 仍无 Refresh**
- [ ] **Step 4: 测试 PASS；映射 auth「刷新轮换」**
- [ ] **Step 5: `./scripts/docs-check`**
