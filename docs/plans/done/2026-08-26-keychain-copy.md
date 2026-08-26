# 完成记录：钥匙串文案诚实标明

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-26 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-26-keychain-copy.md](../2026-08-26-keychain-copy.md) |

## 退出标准

- [x] 浏览器预览钥匙串行不写本机钥匙串，说明 Refresh 不进 Web Storage
- [x] 登录脚注不写只写入系统钥匙串
- [x] Tauri 下钥匙串行仍标明本机钥匙串
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
119 passed

./scripts/docs-check
docs-check 通过（163 个 Markdown 文件）。
```

未改令牌存放。未在真实 Tauri 窗口手工点开设置。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：钥匙串文案已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 冲突时「保留本地」手动选项
- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
