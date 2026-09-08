# 项目状态

| 字段 | 值 |
|---|---|
| 状态 | 现行（活文档，只写今天为真的事） |
| 更新日期 | 2026-09-08（管理端范围代码与本地全端验收完成，外部验收和提交门禁未完成） |

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
| M6 管理台基础范围 | 旧范围完成；完整后台本地已验、外部及提交未完成 | [旧 M6 证据](done/2026-08-23-m6-admin-console.md)；[最终联调记录](2026-09-08-admin-acceptance.md) |
| M7 合同补齐 | 完成 | [done/2026-08-23-m7-contract-gaps.md](done/2026-08-23-m7-contract-gaps.md) |
| M8 设置对齐 | 完成 | [done/2026-08-23-m8-settings-ia.md](done/2026-08-23-m8-settings-ia.md) |
| M9 浏览器工作台与 MCP | 完成 | [done/2026-08-24-m9-web-and-mcp.md](done/2026-08-24-m9-web-and-mcp.md) |

## 当前可执行的下一步

当前用户指定任务为[完整管理端](2026-09-07-admin-complete.md)，原型 12 页和新增范围的实现、本地验收及剩余外部条件见[最终联调记录](2026-09-08-admin-acceptance.md)。不再把已完成代码的邀请、注册邮件、引用迁移、文本 AI 与通知列为待开发；真实凭据验收与 WorkLog 提交门禁仍未完成。以下旧队列记录仅表示各自切片，不得据此宣布生产或全部外部验收通过。

1. 历史实现计划已关闭，但实际功能验收未完成。当前队首：[功能复审与修复](2026-09-05-functional-repair.md)。外部依赖见 [deferred.md](deferred.md)。
2. 工作台壳层接线已关闭。证据：[done/2026-08-29-workbench-shell-wiring.md](done/2026-08-29-workbench-shell-wiring.md)。
3. 手动配置代理已关闭。证据：[done/2026-08-27-manual-proxy.md](done/2026-08-27-manual-proxy.md)。
4. 匿名下载统计已关闭。证据：[done/2026-08-27-anonymous-download-stats.md](done/2026-08-27-anonymous-download-stats.md)。
5. 自动同步收藏与发布草稿已关闭。证据：[done/2026-08-27-auto-sync-queue.md](done/2026-08-27-auto-sync-queue.md)。
6. 仅 Wi-Fi 下同步图片已关闭。证据：[done/2026-08-27-wifi-image-sync.md](done/2026-08-27-wifi-image-sync.md)。
7. 仓库入口阶段已标明 M9 与已接通能力。证据：[done/2026-08-27-readme-status.md](done/2026-08-27-readme-status.md)。
8. 浏览器账单入口已关闭。证据：[done/2026-08-27-web-billing.md](done/2026-08-27-web-billing.md)。
9. 冲突时保留本地已关闭。证据：[done/2026-08-26-keep-local-conflict.md](done/2026-08-26-keep-local-conflict.md)。
10. 钥匙串文案已在浏览器预览标明不进本机钥匙串。证据：[done/2026-08-26-keychain-copy.md](done/2026-08-26-keychain-copy.md)。
11. 网络页同步状态已标明手动立即同步。证据：[done/2026-08-26-sync-status-copy.md](done/2026-08-26-sync-status-copy.md)。
12. 设置冲突处理已标明较新者胜。证据：[done/2026-08-25-sync-conflict-copy.md](done/2026-08-25-sync-conflict-copy.md)。
13. 桌面包版本已与 Tauri 构建对齐。证据：[done/2026-08-25-desktop-version.md](done/2026-08-25-desktop-version.md)。
14. Checkout webhook 入账已关闭。证据：[done/2026-08-25-stripe-webhook.md](done/2026-08-25-stripe-webhook.md)。
15. 预发账单已关闭。证据：[done/2026-08-25-preview-billing.md](done/2026-08-25-preview-billing.md)。
16. Windows / Linux 开机启动与托盘已关闭。证据：[done/2026-08-25-win-linux-prefs.md](done/2026-08-25-win-linux-prefs.md)。
17. 自动更新安装已关闭。证据：[done/2026-08-25-auto-update.md](done/2026-08-25-auto-update.md)。
18. 个人库云同步已关闭。证据：[done/2026-08-25-library-sync.md](done/2026-08-25-library-sync.md)。
19. 账号与广场剩余行已关闭。证据：[done/2026-08-25-account-surface.md](done/2026-08-25-account-surface.md)。
20. 客户端 Google / GitHub 登录已接到桌面、浏览器工作台与管理台。证据：[done/2026-08-25-oauth-clients.md](done/2026-08-25-oauth-clients.md)。
21. 预发后端已接到本机 `promptark` 库。证据：[done/2026-08-25-postgres-backend.md](done/2026-08-25-postgres-backend.md)。
22. 本仓库 `backend/` 是预发，不是生产。不要声称公开下载或上架商店。
23. 启动器仍不请求广场或管理接口。

## 仓库事实

- 应用代码：`desktop/` 本地工作台 + 独立启动器；`mcp/` 本机 MCP；`web/` 浏览器工作台；`backend/` 本机会话 / 广场 / 发布 / 审核（Postgres 库 `promptark`）；`admin-web/` 独立管理端
- 验证：见 [如何在本机工作](../how-to/local-dev.md)
- docs-check：本地可通过
- 旧仓库：只读参考，不是本仓库状态
