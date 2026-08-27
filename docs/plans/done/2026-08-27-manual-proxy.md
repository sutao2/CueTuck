# 完成记录：手动配置代理

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-27 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-27-manual-proxy.md](../2026-08-27-manual-proxy.md) |

## 退出标准

- [x] 设置可填写 HTTP/HTTPS 代理，空则跟随系统，该行不写尚未提供
- [x] 合法地址写入本机设置；本机 Tauri 请求走该代理
- [x] 非法地址不保存；不支持 SOCKS
- [x] 文案标明浏览器预览不走该代理
- [x] 桌面测试、Tauri `http::` 测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
148 passed

cd desktop/src-tauri && cargo test http::
4 passed

./scripts/docs-check
docs-check 通过（177 个 Markdown 文件）。
```

关闭时 docs-check 文件数含本完成记录。完整产品逐步计划已空。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：手动配置代理已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端
- Windows NSIS 验证：GitHub 账号账单失败，作业无法启动
