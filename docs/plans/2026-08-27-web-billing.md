# 浏览器账单入口

> **给 Agent：** 不是新里程碑。预发账单 API 与桌面账号页已接通。浏览器工作台已登录后须能查状态、兑换、仅在测试 Checkout 地址时跳转。不得声称商店上架或公开售卖。不得改后端入账。

**Goal:** 浏览器已登录可查账单并兑换；无密钥标明支付未开通；有测试 Checkout 才打开 Stripe；不声称上架。

**Architecture:** `web/` 调已有 `/v1/billing/*`。未登录不请求。Checkout 只打开 `https://checkout.stripe.com/` 地址。

**Tech Stack:** web 工作台、billing 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 浏览器可查账单并兑换

**Files:** `web/src/` · `docs/specs/billing/spec.md`

- [x] **Step 1: Write the failing test**（已登录未开通不得打开结账；有测试 Checkout 才打开；不声称上架）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: web 账单入口接到已有 API**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/billing/spec.md`

- [x] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [x] **Step 2: 关闭 done 记录；队列回到 deferred**
- [x] **Step 3: `./scripts/docs-check` 并提交**
