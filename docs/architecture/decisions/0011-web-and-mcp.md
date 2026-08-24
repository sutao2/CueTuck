# 11. 浏览器工作台与本机 MCP

- Status: accepted
- Date: 2026-08-24
- 关联：[提案](../../changes/m9-web-and-mcp/proposal.md) · [web](../../specs/web/spec.md) · [mcp](../../specs/mcp/spec.md)

## Context

人要求 Web 端（Notion 式全适应、先不做原生移动端），以及给 Codex 等工具查询提示词的入口。可选形态包括 ChatGPT 插件、编辑器扩展、浏览器扩展、MCP 服务。浏览器无法打开桌面 SQLite；云同步尚未立项。宪法原「完整 Web 个人库不在第一期」需收窄为「不做云端同步个人库 / 原生移动端」，以便 M9 交付浏览器工作台。

## Decision

Agent 入口采用 **MCP stdio 服务**：只读本机 `promptark.sqlite`，工具为搜索、读取、变量渲染。不先做 ChatGPT 插件或 IDE 专有扩展；那些以后可以包同一 MCP。浏览器工作台是独立 `web/` SPA，流体桌面布局，不进桌面包，不声称与桌面库已同步。原生移动端仍不做。

## Consequences

- 新增 `docs/specs/web/spec.md` 与 `docs/specs/mcp/spec.md`。
- 宪法与 PRD 改为：本期不做原生移动端与云端同步个人库；M9 做浏览器工作台与 MCP。
- MCP 不请求广场或管理接口。启动器合同不变。
- 云同步、OAuth、商店上架仍另开变更。
