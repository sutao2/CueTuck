# 项目状态

| 字段 | 值 |
|---|---|
| 状态 | 现行（活文档，只写今天为真的事） |
| 更新日期 | 2026-08-23（M6 Task 4 用户列表） |

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
| M6 运营后台 | 进行中 | [2026-08-23-m6-admin-console.md](2026-08-23-m6-admin-console.md)；Task 4 完成 |

## 当前可执行的下一步

1. M6 Task 4 已有用户只读列表。下一步是 Task 5：运行时开关与桌面包隔离。
2. 本仓库 `backend/` 是预发，不是生产。管理端不进桌面包。
3. 启动器仍不请求广场或管理接口。

## 仓库事实

- 应用代码：`desktop/` 本地工作台 + 独立启动器；`backend/` 本机会话 / 广场 / 发布 / 审核（内存）；`admin-web/` 独立管理端
- 验证：见 [如何在本机工作](../how-to/local-dev.md)
- docs-check：本地可通过
- 旧仓库：只读参考，不是本仓库状态
