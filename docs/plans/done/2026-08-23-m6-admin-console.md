# 完成记录：M6 运营后台

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 里程碑 | M6 |
| 计划 | [2026-08-23-m6-admin-console.md](../2026-08-23-m6-admin-console.md) |

## 退出标准

- [x] 管理规格有测试或 QA
- [x] 客户端包不含管理代码
- [x] done 记录 + status 更新

## 命令与结果

```text
cd desktop && npm test
Test Files  18 passed (18)
Tests  58 passed (58)

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked
28 passed; 0 failed; 1 ignored
（ignored = search_ten_thousand_prompts_bench）

cd admin-web && npm test
Test Files  2 passed (2)
Tests  5 passed (5)

cd backend && unset CARGO_TARGET_DIR && cargo test --locked
15 passed; 0 failed; 1 ignored
（ignored = session_api_postgres_container，无 Docker）

./scripts/docs-check
docs-check 通过（99 个 Markdown 文件）。
```

合同：[ADR 0009](../../architecture/decisions/0009-m6-admin-console.md)、[OpenAPI](../../reference/openapi/admin.yaml)。管理员审核 pending、只读用户列表、`square_public` 开关。`admin-web` 独立于桌面包；Refresh 不进 Web Storage。本仓库 `backend/` 是预发，不声称生产。未打发行标签，无商店包、无公开下载声明。

## 文档

- 更新的规格：admin（审核 / 用户 / 设置 / 隔离已映射）
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- OpenAPI `GET /v1/admin/me` 尚未实现
- 已登录后的收藏 PUT / 取消收藏
- Access / Refresh 轮换与重启后恢复会话
- 密码 KDF（当前 SHA-256）；`PROMPTARK_ALLOW_DEV_USER` 门闩
- Postgres + Testcontainers（无 Docker 时保持 ignore）
- OAuth、计费、完整 Web 个人库
- Playwright CI、签名、商店上架、Windows / Linux 安装包
