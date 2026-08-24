# 变更提案：M9 浏览器工作台与本机 MCP

| 字段 | 值 |
|---|---|
| 状态 | 接受 |
| 日期 | 2026-08-24 |
| 影响规格 | [web](../../specs/web/spec.md) · [mcp](../../specs/mcp/spec.md) |

人要求浏览器端（Notion 式全适应、先不做原生移动端），以及给 Codex 等 Agent 用的提示词查询入口。本提案视为已接受。决定以 [ADR 0011](../../architecture/decisions/0011-web-and-mcp.md) 为准。

实现按 [M9 计划](../../plans/2026-08-24-m9-web-and-mcp.md)。

## 为什么

现行产品是桌面窗口 + 预发广场 API。浏览器里的 Vite 预览不是可交付的 Web 工作台。Agent（Codex / Claude / Cursor）没有标准入口读本机提示词。ChatGPT 插件、编辑器专有插件各自一套，重复且不能复用。

## 做什么

1. 独立浏览器工作台：流体布局、侧栏可收起，在常见桌面浏览器宽度下可用。不发移动端安装包，不为手机单独做一套产品。
2. 给 Agent 的入口是 **MCP stdio 服务**：搜索、读取、按变量渲染本机提示词。
3. MCP 只读本机 SQLite，不请求广场或管理接口。
4. 浏览器工作台不假装能打开桌面那份 SQLite；未做云同步前，两边库可以不同。

## 不做什么

- 不做原生 iOS / Android
- 不做 ChatGPT 插件、VS Code 专有插件（需要时再包一层 MCP）
- 不实现云同步引擎、OAuth、自动更新安装
- 不把 Web 或 MCP 打进桌面安装包
- 不让启动器或 MCP 请求广场

## 依赖

M8 已关闭。本仓库 `backend/` 仍是预发。
