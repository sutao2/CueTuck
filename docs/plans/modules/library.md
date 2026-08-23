# 模块：本地提示词

| 字段 | 值 |
|---|---|
| 里程碑 | M2 |
| 规格 | [library](../../specs/library/spec.md) |
| 分步计划 | [2026-08-22-m2-local-workbench.md](../2026-08-22-m2-local-workbench.md) |

## 完成时必须为真

- 可新建、编辑、软删除、按标题/正文搜索
- 变量只来自正文 `{{名称}}`
- 成功复制后 `use_count` + 1

## 依赖

工作台壳、SQLite schema 中的 `prompts`。
