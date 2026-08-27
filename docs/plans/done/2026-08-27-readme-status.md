# 完成记录：仓库入口阶段诚实标明

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-27 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-27-readme-status.md](../2026-08-27-readme-status.md) |

## 退出标准

- [x] README 标明 M9 已关闭，不把已接通的同步、OAuth、自动更新写成未接通
- [x] 仍标明无商店包
- [x] CLAUDE 不把同步、更新、账单列为未接通示例
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
122 passed

./scripts/docs-check
docs-check 通过（169 个 Markdown 文件）。
```

未改应用行为。未改宪法第 13 条里「按 ADR 0014 排队实现」的历史表述。

## 文档

- 更新的规格：documentation
- 更新的 INDEX：是
- status.md：仓库入口阶段已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
