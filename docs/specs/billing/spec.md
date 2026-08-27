# 账单

| 字段 | 值 |
|---|---|
| 状态 | 已指定；预发 status、兑换、测试 Checkout、webhook 入账与浏览器入口已接通 |
| 关联 | [ADR 0014](../../architecture/decisions/0014-full-product.md) |

## Purpose

预发环境可查询订阅状态并兑换码。不是商店上架，不是生产扣款。

## Requirements

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
| 未开通不得写成 Pro | `backend/tests/billing.rs` unsigned_status_is_not_pro_when_payment_is_unconfigured |
| 兑换成功 | `backend/tests/billing.rs` valid_redeem_code_marks_account_pro_and_cannot_be_reused |
| 无测试密钥不得结账 | `backend/tests/billing.rs` checkout_requires_test_secret_and_does_not_mark_pro |
| 签名通过才入账 | `backend/tests/billing.rs` signed_checkout_webhook_marks_pro_and_rejects_invalid_signatures |
| 浏览器未开通不得打开结账 | `web/src/WebApp.spec.js` shows unpaid billing as 支付未开通 and does not open checkout |
| 浏览器有测试 Checkout 才打开 | `web/src/WebApp.spec.js` opens Stripe checkout only when a test checkout url is returned |
| 浏览器兑换成功 | `web/src/WebApp.spec.js` redeems a code without claiming a store listing |
