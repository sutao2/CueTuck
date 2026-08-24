# 设计：M9 浏览器工作台与本机 MCP

| 字段 | 值 |
|---|---|
| 状态 | 目标 |
| 关联 | [提案](proposal.md) · [ADR 0011](../../architecture/decisions/0011-web-and-mcp.md) |

## 浏览器工作台

独立 `web/`，Vue 3 + Vite，不进桌面包。布局跟现有工作台同一设计源，宽度自适应：宽屏侧栏 + 内容区，窄桌面侧栏可收起。不把触控手机当成第一布局。数据层沿用桌面在浏览器里的内存库 + 预发广场 API，不直连 `promptark.sqlite`。

## MCP

独立 `mcp/` Rust stdio 进程。环境变量 `PROMPTARK_LIBRARY_DIR` 指向含 `promptark.sqlite` 的目录（与桌面 `app_data_dir` 相同）。工具：

- `search_prompts`：按标题或正文本地查询，不含已软删
- `get_prompt`：按 id 返回标题与正文
- `render_prompt`：用与桌面相同的 `{{名称}}` 规则渲染；未填保留 `{{名称}}`

缺库文件时明确报错，不编造条目。不发起 HTTP。

## Codex 配置（说明，不是实现）

宿主用 stdio 拉起 `promptark-mcp`。VS Code / Claude Desktop 同一二进制。不另做插件商店包。
