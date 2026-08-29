# 完成记录：工作台壳层接线

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-29 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-29-workbench-shell-wiring.md](../2026-08-29-workbench-shell-wiring.md) |

## 退出标准

- [x] 广场模型下拉请求 `?model=`，选项来自本机目录与条目
- [x] 本地「最近」按 `last_used_at`；「收藏」按本机星标；右键只提供已有动作
- [x] 显示模型标签、默认模型与目录接到编辑器和卡片
- [x] 变量智能建议只用本机词典，不把正文送到本机以外
- [x] 界面语言 English 切换壳层与设置导航
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
169 passed

./scripts/docs-check
docs-check 通过（179 个 Markdown 文件）。
```

关闭时完整产品逐步计划再次只剩 deferred。

## 文档

- 更新的规格：square、workbench、library、settings
- 更新的 INDEX：是
- status.md：工作台壳层接线已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 商店上架、生产托管、原生移动端、Windows NSIS
- 设置页正文尚未整页翻译；启动器仍只搜本地
