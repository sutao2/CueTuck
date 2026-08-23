# 完成记录：M7 合同补齐

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 里程碑 | M7 |
| 计划 | [2026-08-23-m7-contract-gaps.md](../2026-08-23-m7-contract-gaps.md) |

## 退出标准

- [x] 已登录收藏 / 取消收藏有测试映射
- [x] 令牌轮换有测试映射
- [x] `GET /v1/admin/me` 有测试映射
- [x] done 记录 + status 更新

## 命令与结果

```text
cd desktop && npm test -- --run
Test Files  18 passed (18)
Tests  62 passed (62)

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked
29 passed; 0 failed; 1 ignored
（ignored = search_ten_thousand_prompts_bench）

cd backend && unset CARGO_TARGET_DIR && cargo test --locked
21 passed; 0 failed; 1 ignored
（ignored = session_api_postgres_container，无 Docker）

./scripts/docs-check
docs-check 通过（109 个 Markdown 文件）。
```

合同：[ADR 0010](../../architecture/decisions/0010-m7-contract-gaps.md)。已登录收藏是账号关系，不写 `source=downloaded`。`POST /v1/session/refresh` 轮换后旧对失效；新 Refresh 只进钥匙串。`GET /v1/admin/me` 管理员 200，普通 Access 拒绝。本仓库 `backend/` 是预发。未打发行标签。

## 文档

- 更新的规格：admin / square / auth
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 设置弹窗对齐原型十类（M8）
- 密码 KDF、Postgres、OAuth、自动更新安装
- 商店包与公开下载声明
