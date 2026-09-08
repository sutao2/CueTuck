# 账单

| 字段 | 值 |
|---|---|
| 状态 | 已指定；预发 status、兑换、测试 Checkout、webhook 入账与浏览器入口已接通 |
| 关联 | [ADR 0014](../../architecture/decisions/0014-full-product.md) |

## Purpose

预发环境可查询订阅状态并兑换码。不是商店上架，不是生产扣款。

## Requirements

### Requirement: 显式 mock 支付

仅后端启动时设置 `PROMPTARK_BILLING_MOCK=1` 才启用 mock。状态返回 `mock` 和 `mock_pro`，`pro` 始终是真实权益。Postgres 运行态模拟结果和订单按账号保存到独立 mock 表，重启保留；无数据库的内存测试态重启清空。MUST NOT 调用 Stripe 或写真实 Pro。mock 时原真实兑换/webhook 拒绝；测试码使用独立 `/v1/billing/mock/redeem` 路径，不访问真实兑换表。

### Requirement: 模拟运营隔离

owner/admin MUST 能查看模拟订单和权益、带原因调整、生成和启停测试码批次、查看使用记录；所有列表显著标记 Mock，无收费金额或真实付款凭证。幂等请求与订单/权益/成功审计同事务；测试码明码只在生成成功时一次展示，列表/审计不能恢复明码。

#### Scenario: 模拟重试与测试码额度

- GIVEN 重复请求或测试码仅剩一次额度
- WHEN 重试订单或多个账号并发兑换
- THEN 幂等请求不重复记账，同码最多额度内成功、同账号不能重复使用；过期/停用拒绝
- AND 不改变真实 accounts.pro，模拟关闭后不能通过直接 API 调整或兑换，历史仍可只读查看

#### Scenario: 模拟支付与重置

- GIVEN 已登录且后端启用 mock
- WHEN Checkout 请求携带 `mock_outcome` 为 success、failure、cancel 或 reset
- THEN 分别模拟成功、失败、取消或重置，后两种失败/取消不改变已有模拟权益
- AND 界面始终显示 Mock 标记，不打开外部支付、不改变真实权益；重置只清除此账号模拟状态

#### Scenario: mock 不可越权或隐式开启

- GIVEN 未登录、另一账号或 mock 关闭
- WHEN 请求模拟操作或读取状态
- THEN 未登录拒绝、另一账号看不到他人模拟权益、关闭时模拟请求拒绝
- AND 非预期 HTTP 错误显示失败，不能当作未订阅成功响应

### Requirement: 状态诚实

`GET /v1/billing/status` MUST 返回当前账号是否 Pro。未配置支付密钥时 MUST 标明支付未开通，MUST NOT 把未付费写成已付费。

#### Scenario: 未开通不得写成 Pro

- GIVEN 预发未配置支付密钥
- WHEN 普通账号查询账单状态
- THEN `pro` 为 false
- AND 说明支付未开通

### Requirement: 兑换

有效兑换码 MUST 把该账号标为 Pro。作废码 MUST 失败且不改状态。预发可用 `PROMPTARK_REDEEM_CODES`（逗号分隔）写入未使用码。

`POST /v1/billing/checkout` MUST 仅在配置了 `sk_test_` 密钥时给出 Checkout 地址。未配置支付密钥时 MUST 说明支付未开通且 MUST NOT 给出结账地址。生产密钥 MUST NOT 在预发启动扣款。Checkout 本身 MUST NOT 把账号写成 Pro。

### Requirement: Checkout 入账

`POST /v1/billing/webhook` MUST 校验 Stripe 签名。`checkout.session.completed` 且 `payment_status` 为 paid、`client_reference_id` 为账号时 MUST 标为 Pro。签名无效或未配置 webhook 密钥时 MUST 失败且 MUST NOT 改状态。

#### Scenario: 兑换成功

- GIVEN 预发生成一条未使用兑换码
- WHEN 已登录用户提交该码
- THEN 状态为 Pro
- AND 再次提交同一码失败

#### Scenario: 无测试密钥不得结账

- GIVEN 预发未配置 Stripe 测试密钥
- WHEN 已登录用户请求 Checkout
- THEN 说明支付未开通
- AND 不给出结账地址
- AND 账号仍不是 Pro

#### Scenario: 签名通过才入账

- GIVEN 预发配置了 webhook 密钥
- WHEN Stripe 送达有效签名的 `checkout.session.completed`
- THEN 该账号为 Pro
- AND 无签名或错签名失败且状态不变

### Requirement: 浏览器入口

浏览器工作台已登录后 MUST 能查询账单状态并兑换。未开通时 MUST 标明支付未开通且 MUST NOT 打开结账。仅当返回 `https://checkout.stripe.com/` 测试地址时 MUST 打开该地址。MUST NOT 声称商店上架或公开售卖。

#### Scenario: 浏览器未开通不得打开结账

- GIVEN 用户已登录且支付未开通
- WHEN 打开浏览器工作台并点前往支付
- THEN 标明支付未开通
- AND 不打开结账
- AND 不声称已从商店上架

#### Scenario: 浏览器兑换成功

- GIVEN 用户已登录且提交有效兑换码
- WHEN 在浏览器工作台兑换
- THEN 状态为 Pro
- AND 不声称已从商店上架

#### Scenario: 浏览器有测试 Checkout 才打开

- GIVEN 用户已登录且返回 Stripe 测试 Checkout 地址
- WHEN 点前往支付
- THEN 打开该地址
- AND 账号仍不是 Pro
- AND 不声称已从商店上架

## 测试映射

| 场景 | 测试 |
|---|---|
| 模拟支付与重置 / mock 不可越权 | `backend/tests/billing.rs` mock 系列、`billing::mock_tests`；桌面/Web mock 组件测试和账单请求失败测试 |
| 未开通不得写成 Pro | `backend/tests/billing.rs` unsigned_status_is_not_pro_when_payment_is_unconfigured |
| 兑换成功 | `backend/tests/billing.rs` valid_redeem_code_marks_account_pro_and_cannot_be_reused |
| 无测试密钥不得结账 | `backend/tests/billing.rs` checkout_requires_test_secret_and_does_not_mark_pro |
| 签名通过才入账 | `backend/tests/billing.rs` signed_checkout_webhook_marks_pro_and_rejects_invalid_signatures |
| 浏览器未开通不得打开结账 | `web/src/WebApp.spec.js` shows unpaid billing as 支付未开通 and does not open checkout |
| 浏览器有测试 Checkout 才打开 | `web/src/WebApp.spec.js` opens Stripe checkout only when a test checkout url is returned |
| 浏览器兑换成功 | `web/src/WebApp.spec.js` redeems a code without claiming a store listing |
