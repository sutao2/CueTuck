# 完成记录：M5 在线广场

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 里程碑 | M5 |
| 计划 | [2026-08-23-m5-online-square.md](../2026-08-23-m5-online-square.md) |

## 退出标准

- [x] 离线时本地仍 100% 可用
- [x] 下载与收藏行为分离
- [x] OpenAPI 或等价合同在 INDEX 中
- [x] done 记录 + status 更新

## 命令与结果

```text
cd desktop && npm test
Test Files  16 passed (16)
Tests  55 passed (55)

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked
28 passed; 0 failed; 1 ignored
（ignored = search_ten_thousand_prompts_bench）

cd backend && unset CARGO_TARGET_DIR && cargo test --locked
7 passed; 0 failed; 1 ignored
（ignored = session_api_postgres_container，无 Docker）

./scripts/docs-check
docs-check 通过（91 个 Markdown 文件）。
```

合同：[ADR 0008](../../architecture/decisions/0008-m5-backend-contract.md)、[OpenAPI](../../reference/openapi/square.yaml)。登录是邮箱密码；Refresh 只进钥匙串。启动器搜索不请求广场。未打桌面发行标签，无商店包、无公开下载声明。

## 文档

- 更新的规格：square / auth / publish（M5 场景已映射）
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 已登录后的收藏 PUT / 取消收藏
- Access / Refresh 轮换与重启后恢复会话
- 密码 KDF（当前 SHA-256）；`PROMPTARK_ALLOW_DEV_USER` 门闩
- Postgres + Testcontainers（无 Docker 时保持 ignore）
- 管理后台（M6）；QQ / LinuxDo / Google
- Playwright CI、签名、商店上架、Windows / Linux 安装包
