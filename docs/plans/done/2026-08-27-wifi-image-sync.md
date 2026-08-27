# 完成记录：仅 Wi-Fi 下同步图片

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-27 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-27-wifi-image-sync.md](../2026-08-27-wifi-image-sync.md) |

## 退出标准

- [x] 设置可开关「仅 Wi-Fi 下同步图片」，默认关闭，该行不写尚未提供
- [x] 打开且网络不是 Wi-Fi（含无法判定）时立即同步跳过本机封面，仍推送标题与正文
- [x] 跳过时不把远端已有封面用空封面覆盖
- [x] Wi-Fi 或开关关闭时仍推送本机封面
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
127 passed

./scripts/docs-check
docs-check 通过（171 个 Markdown 文件）。
```

关闭时 docs-check 文件数含本完成记录。未接 MinIO 独立资源上传。浏览器预览无法判定 Wi-Fi 时按非 Wi-Fi 跳过封面。

## 文档

- 更新的规格：sync、settings
- 更新的 INDEX：是
- status.md：仅 Wi-Fi 同步图片已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 自动同步收藏与发布草稿队列
- 匿名下载统计、手动配置代理
- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
