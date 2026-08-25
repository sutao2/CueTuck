# 完成记录：个人库云同步

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-library-sync.md](../2026-08-25-library-sync.md) |

## 退出标准

- [x] `GET/PUT /v1/library/changes` 需登录；PUT 后 GET `since=` 拉到同一 id 与正文
- [x] 已登录立即同步推拉本机 SQLite；未登录打开登录且不出现已同步
- [x] 默认较新 `updated_at` 胜；浏览器登录后可见同一标题且不出现「已写入本机 SQLite」
- [x] 启动器与 MCP 仍只碰本机 SQLite
- [x] 桌面 / web / backend 测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
98 passed

cd web && npm test
16 passed

cd backend && unset CARGO_TARGET_DIR && cargo test --locked --offline
32 lib + library_changes + me + publications_mine passed

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked
36 passed (1 ignored)

./scripts/docs-check
docs-check 通过（150 个 Markdown 文件）。
```

未在真实预发账号与第二台机器上做手工同步 smoke。覆盖率 80%/70% 未实测。用户选择保留本地的冲突选项未做。

## 文档

- 更新的规格：sync、settings、web
- 更新的 INDEX：是
- status.md：个人库云同步已关闭；下一步自动更新安装

## 未做 / 下一里程碑带走

- 自动更新、Windows/Linux 偏好、预发账单
- 商店上架、生产托管、Windows NSIS 额度
- 冲突时「保留本地」手动选项
