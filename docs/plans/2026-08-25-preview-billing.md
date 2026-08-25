# 预发账单

> **给 Agent：** 不是新里程碑。先读 [billing 规格](../specs/billing/spec.md)。无支付密钥不得把账号写成 Pro。不得声称商店上架。队首关闭后再改为现行并补逐步任务。

**Goal:** 预发可查询账单状态并用兑换码升 Pro；Stripe 仅在测试密钥存在时跳转。

**Architecture:** `/v1/billing/status`、兑换码表。Checkout 可选。

**Tech Stack:** 本仓库 backend、Postgres `promptark`。

## Global Constraints

- 不得把未付费写成已付费。
- 不得改 README 声称已上架或公开售卖。
- 未成为队首禁止写应用代码。
