# Stripe Checkout 入账

> **给 Agent：** 不是新里程碑。先读 [billing 规格](../specs/billing/spec.md)。无有效签名不得把账号写成 Pro。不得声称商店上架。

**Goal:** 预发用 Stripe webhook 把已完成的测试 Checkout 记成 Pro；签名无效或未配置密钥时保持未付费。

**Architecture:** `POST /v1/billing/webhook` 校验 `Stripe-Signature`；`checkout.session.completed` 用 `client_reference_id` 入账。Checkout 本身仍不改 Pro。

**Tech Stack:** 本仓库 backend、Postgres `promptark`。

## Global Constraints

- 不得把未付费写成已付费。
- 不得改 README 声称已上架或公开售卖。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 签名通过才入账

**Files:** `backend/src/` · `docs/reference/openapi/` · `docs/specs/billing/spec.md`

- [x] **Step 1: Write the failing test**（有效签名的 `checkout.session.completed` 升 Pro；无签名或错签名失败且不改状态）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: webhook 接口与签名校验；无密钥不得入账**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/billing/spec.md`

- [x] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [x] **Step 2: 关闭 done 记录；队列回到 deferred**
- [x] **Step 3: `./scripts/docs-check` 并提交**
