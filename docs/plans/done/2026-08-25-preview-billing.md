# 完成记录：预发账单与兑换

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-preview-billing.md](../2026-08-25-preview-billing.md) |

## 退出标准

- [x] 未配置支付密钥时 `GET /v1/billing/status` 的 `pro` 为 false，并说明支付未开通
- [x] 有效兑换码升 Pro；同一码再提交失败且不改其他账号状态
- [x] 仅 `sk_test_` 密钥给出 Checkout；无密钥说明未开通；生产密钥不在预发启动扣款；Checkout 不把账号写成 Pro
- [x] 设置账号页可查账单、兑换、前往支付；不声称公开售卖
- [x] 后端、桌面测试与 docs-check 通过

## 命令与结果

```text
cd backend && unset CARGO_TARGET_DIR && cargo test --locked --offline --test billing
3 passed

cd desktop && npm test
111 passed

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked --offline --lib
38 passed (1 ignored)

./scripts/docs-check
docs-check 通过（153 个 Markdown 文件）。
```

Checkout 成功不会把账号写成 Pro；Stripe webhook 入账未做。未用真实 `sk_test_` 密钥做手工结账 smoke。

## 文档

- 更新的规格：billing
- 更新的 INDEX：是
- status.md：预发账单已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub Actions 因账号账单失败无法启动作业，与仓库是否公开无关
- Stripe webhook 入账
