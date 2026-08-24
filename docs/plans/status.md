# 项目状态

| 字段 | 值 |
|---|---|
| 状态 | 现行（活文档，只写今天为真的事） |
| 更新日期 | 2026-08-24（M9 进行中；先 MCP 后 Web） |

禁止在本文件预写「已完成」。完成时改勾选并链到 `done/` 记录。

## 总览

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M0 文档先行 | 完成 | [done/2026-08-22-m0-documentation.md](done/2026-08-22-m0-documentation.md) |
| M1 桌面骨架 | 完成 | [done/2026-08-22-m1-desktop-skeleton.md](done/2026-08-22-m1-desktop-skeleton.md) |
| M2 本地工作台 | 完成 | [done/2026-08-23-m2-local-workbench.md](done/2026-08-23-m2-local-workbench.md) |
| M3 启动器对齐 | 完成 | [done/2026-08-23-m3-launcher.md](done/2026-08-23-m3-launcher.md) |
| M4 桌面可分发 | 完成 | [done/2026-08-23-m4-desktop-distributable.md](done/2026-08-23-m4-desktop-distributable.md) |
| M5 在线广场 | 完成 | [done/2026-08-23-m5-online-square.md](done/2026-08-23-m5-online-square.md) |
| M6 运营后台 | 完成 | [done/2026-08-23-m6-admin-console.md](done/2026-08-23-m6-admin-console.md) |
| M7 合同补齐 | 完成 | [done/2026-08-23-m7-contract-gaps.md](done/2026-08-23-m7-contract-gaps.md) |
| M8 设置对齐 | 完成 | [done/2026-08-23-m8-settings-ia.md](done/2026-08-23-m8-settings-ia.md) |
| M9 浏览器工作台与 MCP | 进行中 | [2026-08-24-m9-web-and-mcp.md](2026-08-24-m9-web-and-mcp.md) |

## 当前可执行的下一步

1. M9 进行中。当前切片：[浏览器工作台与本机 MCP](2026-08-24-m9-web-and-mcp.md)。先做 MCP，再做独立 `web/`。不做原生移动端。不声称浏览器库与桌面已同步。
2. 本仓库 `backend/` 是预发，不是生产。不要声称公开下载或上架商店。
3. 启动器仍不请求广场或管理接口。

## 仓库事实

- 应用代码：`desktop/` 本地工作台 + 独立启动器；`mcp/` 本机 MCP；`backend/` 本机会话 / 广场 / 发布 / 审核（内存）；`admin-web/` 独立管理端；`web/` 尚未开工
- 验证：见 [如何在本机工作](../how-to/local-dev.md)
- docs-check：本地可通过
- 旧仓库：只读参考，不是本仓库状态
