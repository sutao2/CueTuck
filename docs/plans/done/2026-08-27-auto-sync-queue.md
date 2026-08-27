# 完成记录：自动同步收藏与发布草稿

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-27 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-27-auto-sync-queue.md](../2026-08-27-auto-sync-queue.md) |

## 退出标准

- [x] 设置可开关「自动同步收藏与发布草稿」，默认关闭，该行不写尚未提供
- [x] 打开且已登录时，收藏或发布失败写入本机队列，不写本地副本，不假装已到达服务器
- [x] 立即同步冲刷本账号队列
- [x] 开关关闭时失败不入队
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
135 passed

./scripts/docs-check
docs-check 通过（173 个 Markdown 文件）。
```

关闭时 docs-check 文件数含本完成记录。队列按账号邮箱冲刷。未做匿名下载统计与手动代理。

## 文档

- 更新的规格：settings、square、publish
- 更新的 INDEX：是
- status.md：自动同步收藏队列已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 匿名下载统计、手动配置代理
- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
