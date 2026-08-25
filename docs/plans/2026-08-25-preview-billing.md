# 预发账单

> **给 Agent：** 不是新里程碑。先读 [billing 规格](../specs/billing/spec.md)。无支付密钥不得把账号写成 Pro。不得声称商店上架。

**Goal:** 预发可查询账单状态并用兑换码升 Pro；Stripe 仅在测试密钥存在时跳转。

**Architecture:** `/v1/billing/status`、兑换码表。Checkout 可选。

**Tech Stack:** 本仓库 backend、Postgres `promptark`。

## Global Constraints

- 不得把未付费写成已付费。
- 不得改 README 声称已上架或公开售卖。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 账单状态诚实

**Files:** `backend/src/` · `docs/reference/openapi/` · `docs/specs/billing/spec.md`

- [ ] **Step 1: Write the failing test**（未配置支付密钥时 `GET /v1/billing/status` 的 `pro` 为 false 且说明支付未开通）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 需登录的 status 接口；无密钥不得写成 Pro**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 兑换码

**Files:** `backend/src/` · `docs/specs/billing/spec.md`

- [ ] **Step 1: Write the failing test**（有效码升 Pro；同一码再提交失败且不改状态）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 兑换码表与提交接口**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 3: Stripe 测试结账并关闭

**Files:** `backend/src/` · 设置或 web 账单入口 · `docs/specs/billing/spec.md`

- [ ] **Step 1: Write the failing test**（有测试密钥才给出 Checkout；无密钥说明未开通；关闭本计划）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 仅在测试密钥存在时跳转；关闭 done 记录**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**
