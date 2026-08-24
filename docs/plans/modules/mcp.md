# 本机 MCP 模块

| 字段 | 值 |
|---|---|
| 里程碑 | M9 |
| 规格 | [mcp](../../specs/mcp/spec.md) |

## 完成时必须为真

- stdio MCP 可列出 `search_prompts`、`get_prompt`、`render_prompt`
- 查询本机 `promptark.sqlite`，缺文件时报错
- 未填变量保留 `{{名称}}`
- 不请求广场或管理接口
