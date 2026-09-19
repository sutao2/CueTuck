# 项目状态

| 字段 | 值 |
|---|---|
| 状态 | 现行（活文档，只写今天为真的事） |
| 更新日期 | 2026-09-19（按已提交的发行、部署和能力记录核对） |

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
| M6 管理台基础范围 | 旧范围完成；完整后台已部署，外部验收按具体记录区分 | [旧 M6 证据](done/2026-08-23-m6-admin-console.md)；[部署记录](2026-09-13-compose-deployment.md)；[AI 审核验收](2026-09-18-ai-decisions.md) |
| M7 合同补齐 | 完成 | [done/2026-08-23-m7-contract-gaps.md](done/2026-08-23-m7-contract-gaps.md) |
| M8 设置对齐 | 完成 | [done/2026-08-23-m8-settings-ia.md](done/2026-08-23-m8-settings-ia.md) |
| M9 浏览器工作台与 MCP | 完成 | [done/2026-08-24-m9-web-and-mcp.md](done/2026-08-24-m9-web-and-mcp.md) |

## 当前交付与核查

- 当前公开预览版为 beta.20；版本、双平台包、远程回归、官网切换及签名边界以[发行记录](2026-09-19-beta20-release.md)为准，不代表商店上架或完整正式签名。
- 最近交付的搜索、同步反馈、回收站、备份恢复及同步前版本见[使用体验与可恢复性](2026-09-19-usability-safety.md)。
- 后端、管理台及官网已有线上部署，见[Compose 部署](2026-09-13-compose-deployment.md)与[官网入口](2026-09-15-launcher-website.md)；不能再将整个仓库称为仅本机预发。
- 当前任务为[文档与代码对齐](2026-09-19-doc-code-alignment.md)。旧 M0–M9 完成记录仅证明当时切片，不代表后续所有能力和外部条件均已验收。
- 提交与验证政策统一见[测试门禁](../reference/test-gates.md)；WorkLog 不再是提交前置条件。覆盖率及隔离后端测试要求的落实情况也以该文件为准。
- 外部凭据、平台与发行限制按各专项记录确认；本次文档核查没有重新执行线上或实机验收。

## 仓库事实

- 应用代码：`desktop/` 本地工作台 + 独立启动器；`mcp/` 本机 MCP；`web/` 浏览器工作台；`backend/` 会话 / 广场 / 发布 / 审核 / 翻译（开发与线上实例分别配置）；`admin-web/` 独立管理端
- 验证：见 [如何在本机工作](../how-to/local-dev.md)
- 启动器默认仅搜本地，显式选择后搜索广场；MCP 默认三个本地只读工具，宿主显式启用后增加独立广场工具；均不调用管理接口。合同见[启动器](../specs/launcher/spec.md)与[MCP](../specs/mcp/spec.md)。
- 文档结构检查与代码合同核对的边界见[文档规格](../specs/documentation/spec.md)。
- 旧仓库：只读参考，不是本仓库状态
