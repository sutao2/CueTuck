# 完成记录：匿名下载统计

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-27 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-27-anonymous-download-stats.md](../2026-08-27-anonymous-download-stats.md) |

## 退出标准

- [x] 设置可开关「匿名下载统计」，默认关闭，该行不写尚未提供
- [x] 打开且下载成功后只 POST 条目 id，不含 Authorization、账号或正文
- [x] 关闭时不请求；统计失败不阻断下载；GET 正文不加次数
- [x] 热门按次数降序；未知 id 404
- [x] 桌面测试、后端测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
143 passed

cd backend && cargo test --test download_stats
3 passed

./scripts/docs-check
docs-check 通过（175 个 Markdown 文件）。
```

关闭时 docs-check 文件数含本完成记录。未做手动配置代理。

## 文档

- 更新的规格：settings、square、OpenAPI
- 更新的 INDEX：是
- status.md：匿名下载统计已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 手动配置代理
- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
