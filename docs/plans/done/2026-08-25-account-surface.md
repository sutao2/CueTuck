# 完成记录：账号与广场剩余行

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-account-surface.md](../2026-08-25-account-surface.md) |

## 退出标准

- [x] 下载时保留作者写入本地展示字段，不改正文
- [x] 已登录设置页列出当前账号投稿与状态
- [x] 已登录可保存显示名与简介；未登录不得写入
- [x] 不得出现 QQ / LinuxDo
- [x] 桌面与 backend 测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
94 passed

cd backend && unset CARGO_TARGET_DIR && cargo test --locked --offline
32 lib + me + publications_mine passed

./scripts/docs-check
docs-check 通过（149 个 Markdown 文件）。
```

未在真实预发账号上做手工授权 smoke。覆盖率 80%/70% 未实测。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：账号面已关闭；下一步个人库云同步

## 未做 / 下一里程碑带走

- 个人库云同步、自动更新、Windows/Linux 偏好、预发账单
- 商店上架、生产托管、Windows NSIS 额度
