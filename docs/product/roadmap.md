# 路线图

| 字段 | 值 |
|---|---|
| 状态 | 现行 |
| 关联 | [PRD](prd.md) · [测试门禁](../reference/test-gates.md) |

进度以本文件和 git 标签为准。不另写百分比进度报告。

## 里程碑

| 里程碑 | 目标 | 状态 |
|---|---|---|
| M0 文档先行 | 仓库、索引、宪法、需求、规格、门禁、模板 | 完成 |
| M1 桌面骨架 | Tauri + Vue 壳、本地 SQLite、启动器窗口可唤起 | 完成 |
| M2 本地工作台 | 本地库、分类、合集、使用向导、设置、导入导出 | 完成 |
| M3 启动器对齐 | 旧启动器行为全部移植并接上新本地库 | 完成 |
| M4 桌面可分发 | 本机 smoke、备份恢复、文档与规格 converge | 完成 |
| M5 在线广场 | 改写后端，接广场/登录/发布 | 完成 |
| M6 运营后台 | 独立管理端审核、用户只读、运行时设置 | 完成 |
| M7 合同补齐 | 已登录收藏、令牌轮换、管理员身份 | 完成 |
| M8 设置对齐 | 设置弹窗对齐原型十类；只增不减已有本机设置 | 完成 |
| M9 浏览器工作台与 MCP | 独立 Web 流体布局；本机 MCP 查询提示词 | 完成 |

M9 已关闭。此后不是新里程碑，按 [ADR 0014](../architecture/decisions/0014-full-product.md) 的完整产品切片队列推进。程序计划见 [plans/program.md](../plans/program.md)。逐步任务只写队首，见 [ADR 0006](../architecture/decisions/0006-plan-altitude.md)。

## M0 完成标准

1. `./scripts/docs-check` 通过。
2. 第一期能力均有 `docs/specs/<能力>/spec.md`。
3. 已记录仓库、启动器、本地优先、文档体系、设计源五条 ADR。
4. 仍无业务应用代码。

## M1 完成标准

1. 主窗口能打开并显示空的本地工作台壳。
2. 全局快捷键能显示/隐藏启动器窗口。
3. SQLite 可初始化，启动器能对空库给出空态。
4. 计划中的骨架测试通过。

## 后期闸门（M7）

- M7 已关闭，见 [done/2026-08-23-m7-contract-gaps.md](../plans/done/2026-08-23-m7-contract-gaps.md)
- 管理台不打进桌面安装包
- 不绑定未选定的 QQ / LinuxDo / Google

## 后期闸门（M8）

- M8 已关闭，见 [done/2026-08-23-m8-settings-ia.md](../plans/done/2026-08-23-m8-settings-ia.md)
- 不得删除 JSON 导入导出、库文件备份恢复、浅色/深色、启动器唤起快捷键
- 云同步、自动更新安装、OAuth 不得假装接通

## 后期闸门（M9）

- M9 已关闭，见 [done/2026-08-24-m9-web-and-mcp.md](../plans/done/2026-08-24-m9-web-and-mcp.md)
- 浏览器工作台不进桌面包，不做原生移动端
- MCP 只读本机库，不请求广场
- 未登录的浏览器库不得写成已与桌面 SQLite 同步

## 完整产品队列

队首与顺序见 [program.md](../plans/program.md)。商店上架、生产托管、Windows NSIS 额度见 [deferred.md](../plans/deferred.md)。
