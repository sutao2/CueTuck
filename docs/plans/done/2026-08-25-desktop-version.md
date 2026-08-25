# 完成记录：桌面包版本对齐

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-desktop-version.md](../2026-08-25-desktop-version.md) |

## 退出标准

- [x] `package.json` / `package-lock.json` 与 Cargo / `tauri.conf.json` 同为 `0.1.0`
- [x] 设置更新页展示该版本，不展示占位 `0.0.0`
- [x] 稳定通道 tag 与本机构建相同时检查更新 `available` 为 false
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
113 passed

./scripts/docs-check
docs-check 通过（157 个 Markdown 文件）。
```

未发布 GitHub Release，也未在真实 Tauri 窗口里手工点检查更新。web / admin-web 的 `0.0.0` 不在本切片。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：桌面包版本已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
