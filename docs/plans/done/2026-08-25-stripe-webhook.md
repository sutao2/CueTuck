# 完成记录：Checkout webhook 入账

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-stripe-webhook.md](../2026-08-25-stripe-webhook.md) |

## 退出标准

- [x] 有效签名的 `checkout.session.completed` 把 `client_reference_id` 账号标为 Pro
- [x] 无签名、错签名或未配置 webhook 密钥失败且不改状态
- [x] Checkout 本身仍不把账号写成 Pro
- [x] 后端测试与 docs-check 通过

## 命令与结果

```text
cd backend && unset CARGO_TARGET_DIR && cargo test --locked --offline --test billing
4 passed

cd backend && unset CARGO_TARGET_DIR && cargo test --locked --offline webhook_survives
persistence_tests::webhook_survives_new_appstate_on_postgres passed

cd desktop && npx vitest run src/platform/squareContract.test.js
1 passed

./scripts/docs-check
docs-check 通过（155 个 Markdown 文件）。
```

未对接真实 Stripe Dashboard webhook，也未做生产托管。

## 文档

- 更新的规格：billing
- 更新的 INDEX：是
- status.md：Checkout webhook 已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
