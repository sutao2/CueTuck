# 账单

| 字段 | 值 |
|---|---|
| 状态 | 已指定；预发 status 已接通 |
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

有效兑换码 MUST 把该账号标为 Pro。作废码 MUST 失败且不改状态。

#### Scenario: 兑换成功

- GIVEN 预发生成一条未使用兑换码
- WHEN 已登录用户提交该码
- THEN 状态为 Pro
- AND 再次提交同一码失败

## 测试映射

| 场景 | 测试 |
|---|---|
| 未开通不得写成 Pro | `backend/tests/billing.rs` unsigned_status_is_not_pro_when_payment_is_unconfigured |
| 兑换成功 | 未开始 |
