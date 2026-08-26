# 完成记录：冲突时保留本地

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-26 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-26-keep-local-conflict.md](../2026-08-26-keep-local-conflict.md) |

## 退出标准

- [x] 设置可选较新者胜或保留本地，默认较新者胜
- [x] 保留本地时立即同步不覆盖已有本机正文
- [x] 远端独有条目仍写入
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
121 passed

./scripts/docs-check
docs-check 通过（165 个 Markdown 文件）。
```

未改后端 PUT 较新者胜。未在真实 Tauri 窗口与第二台机器上做手工同步 smoke。

## 文档

- 更新的规格：sync、settings
- 更新的 INDEX：是
- status.md：保留本地冲突已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
