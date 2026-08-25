# 完成记录：设置冲突策略诚实标明

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-sync-conflict-copy.md](../2026-08-25-sync-conflict-copy.md) |

## 退出标准

- [x] 同步页冲突处理标明较新者胜，该行不写尚未提供
- [x] 不出现可选择保留本地的控件
- [x] 自动同步收藏与仅 Wi-Fi 仍标明尚未提供
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
114 passed

./scripts/docs-check
docs-check 通过（159 个 Markdown 文件）。
```

未做冲突时「保留本地」选项。未在真实 Tauri 窗口手工点开设置。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：冲突策略文案已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 冲突时「保留本地」手动选项
- 网络页「同步状态」仍写没有云同步
- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
