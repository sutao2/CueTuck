# 项目状态

| 字段 | 值 |
|---|---|
| 状态 | 现行（活文档，只写今天为真的事） |
| 更新日期 | 2026-08-23（M5 计划 Task 0–5 已落地，尚无 done 记录） |

禁止在本文件预写「已完成」。完成时改勾选并链到 `done/` 记录。

## 总览

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M0 文档先行 | 完成 | [done/2026-08-22-m0-documentation.md](done/2026-08-22-m0-documentation.md) |
| M1 桌面骨架 | 完成 | [done/2026-08-22-m1-desktop-skeleton.md](done/2026-08-22-m1-desktop-skeleton.md) |
| M2 本地工作台 | 完成 | [done/2026-08-23-m2-local-workbench.md](done/2026-08-23-m2-local-workbench.md) |
| M3 启动器对齐 | 完成 | [done/2026-08-23-m3-launcher.md](done/2026-08-23-m3-launcher.md) |
| M4 桌面可分发 | 完成 | [done/2026-08-23-m4-desktop-distributable.md](done/2026-08-23-m4-desktop-distributable.md) |
| M5 在线广场 | 合同已接受 | [ADR 0008](../architecture/decisions/0008-m5-backend-contract.md)；尚无 done 记录 |
| M6 运营后台 | 未开始 | — |

## 当前可执行的下一步

1. M5 计划 Task 0–5 已落地（浏览、匿名下载、收藏登录、发布不锁本地）。尚无 M5 done 记录，不要声称可公开下载或上架商店。启动器仍不请求广场。
2. 覆盖率已写入 [测试门禁](../reference/test-gates.md)：行 80%、分支 70%。
3. 不要在签名未齐时声称可公开下载或上架商店。

## 仓库事实

- 应用代码：`desktop/` 本地工作台 + 独立启动器（搜索、填写、快捷键、粘贴降级）
- 验证：见 [如何在本机工作](../how-to/local-dev.md)
- 后端：`backend/` 本机会话、广场列表/正文、审核提交（内存）
- docs-check：本地可通过
- 旧仓库：只读参考，不是本仓库状态
