# 完成记录：预发后端持久化

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-24-postgres-backend.md](../2026-08-24-postgres-backend.md) |

## 退出标准

- [x] 口令 Argon2id，裸 SHA-256 不能当校验器
- [x] 独立库 `promptark`（不写 Flyway `pl`）；进程换 AppState 后会话/广场/收藏/设置仍在
- [x] Google / GitHub 授权码 API；未配凭据则 providers 为空
- [x] 本机 Postgres / Redis / MinIO 健康检查为真
- [x] `cargo test --locked` 全绿；docs-check 通过

## 命令与结果

```text
cd backend && unset CARGO_TARGET_DIR && cargo test --locked
32 passed; 0 failed; 0 ignored

curl http://127.0.0.1:8787/v1/health
{"minio":true,"postgres":true,"redis":true}

./scripts/docs-check
docs-check 通过
```

覆盖率 80%/70% 未实测，不得写成已达到。

## 文档

- 更新的规格：auth、square、admin、settings
- 更新的 INDEX：是
- status.md：预发后端已接到 `promptark` 库

## 未做 / 下一里程碑带走

- 桌面 / web / admin-web 登录弹窗尚未列出 Google / GitHub
- 云同步、账单、商店上架、自动更新安装
- Windows NSIS 与非 macOS 托盘仍未验证
