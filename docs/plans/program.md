# 程序计划

| 字段 | 值 |
|---|---|
| 状态 | 现行 |
| 关联 | [路线图](../product/roadmap.md) · [状态](status.md) · [ADR 0006](../architecture/decisions/0006-plan-altitude.md) |

一份总图：先做什么、依赖什么、怎样才算过门。不代替各里程碑的逐步计划。

## 顺序与依赖

```text
M0 文档体系
 └─ M1 桌面骨架（壳 + SQLite + 启动器可唤起）
     └─ M2 本地工作台（库 / 分类 / 合集 / 变量 / 设置）
         └─ M3 启动器对齐（旧窗口接到新库）
             └─ M4 桌面可分发
                 └─ M5 在线广场（另立变更：复用或改写后端）
                     └─ M6 运营后台
                         └─ M7 合同补齐
                             └─ M8 设置对齐原型
                                 └─ M9 浏览器工作台与本机 MCP
                                     └─ 关闭后的切片队列（不是新里程碑）
```

M3 可以与 M2 后期并行调研，但不得在新库未稳定时整份移植旧 repository。

## M9 之后的切片队列

详细逐步任务仍一次只执行**队首**。后一份在 INDEX 标「目标」，队首标「现行」。关闭队首后把下一份改成现行。

```text
1–5 已关闭（广场预发缺口 … 预发后端持久化）
6. 客户端接 Google / GitHub（现行）
之后：deferred.md（云同步 / 账单 / 商店等，无逐步任务）
```

| 顺序 | 计划 | INDEX |
|---|---|---|
| 1 已关闭 | [2026-08-24-square-preview-gaps.md](2026-08-24-square-preview-gaps.md) | 归档 |
| 2 已关闭 | [2026-08-24-web-edit-prompt.md](2026-08-24-web-edit-prompt.md) | 归档 |
| 3 已关闭 | [2026-08-24-web-use-wizard.md](2026-08-24-web-use-wizard.md) | 归档 |
| 4 已关闭 | [2026-08-24-web-square-preview.md](2026-08-24-web-square-preview.md) | 归档 |
| 现在 | [2026-08-25-oauth-clients.md](2026-08-25-oauth-clients.md) | 现行 |
| 已关闭 | [2026-08-24-postgres-backend.md](2026-08-24-postgres-backend.md) | 归档 |
| 之后 | [deferred.md](deferred.md) | 目标 |

## 模块归属

| 模块 | 里程碑 | 规格 | 分步计划 |
|---|---|---|---|
| 文档体系 | M0 | [documentation](../specs/documentation/spec.md) | 无应用计划 |
| 工作台壳 | M1 | [workbench](../specs/workbench/spec.md) | [2026-08-22-m1-desktop-skeleton.md](2026-08-22-m1-desktop-skeleton.md) |
| 本地库 | M2 | [library](../specs/library/spec.md) | [2026-08-22-m2-local-workbench.md](2026-08-22-m2-local-workbench.md) |
| 分类 | M2 | [categories](../specs/categories/spec.md) | 同上 |
| 合集 | M2 | [collections](../specs/collections/spec.md) | 同上 |
| 变量向导 | M2 | [variables](../specs/variables/spec.md) | 同上 |
| 设置 | M2 | [settings](../specs/settings/spec.md) | [2026-08-22-m2-local-workbench.md](2026-08-22-m2-local-workbench.md) |
| 启动器 | M3 | [launcher](../specs/launcher/spec.md) | [2026-08-23-m3-launcher.md](2026-08-23-m3-launcher.md) |
| 广场 / 认证 / 发布 | M5 | 对应 specs | [2026-08-23-m5-online-square.md](2026-08-23-m5-online-square.md) |
| 管理台 | M6 | [admin](../specs/admin/spec.md) | [2026-08-23-m6-admin-console.md](2026-08-23-m6-admin-console.md) |
| 合同补齐（收藏写 / 轮换 / me） | M7 | 对应 specs | [2026-08-23-m7-contract-gaps.md](2026-08-23-m7-contract-gaps.md) |
| 设置对齐原型 | M8 | [settings](../specs/settings/spec.md) | [2026-08-23-m8-settings-ia.md](2026-08-23-m8-settings-ia.md) |
| 浏览器工作台 | M9 | [web](../specs/web/spec.md) | [2026-08-24-m9-web-and-mcp.md](2026-08-24-m9-web-and-mcp.md) |
| 本机 MCP | M9 | [mcp](../specs/mcp/spec.md) | 同上 |
| 广场预发缺口 | M5 合同补丁 | [square](../specs/square/spec.md) / [publish](../specs/publish/spec.md) | [2026-08-24-square-preview-gaps.md](2026-08-24-square-preview-gaps.md) |

## 通用完成定义（每个里程碑都要满足）

- 该里程碑规格场景：已实现的有测试映射，未做的仍写「未开始」，不得删场景
- `./scripts/docs-check` 通过
- [status.md](status.md) 已更新
- 有一份 [done](done/README.md) 完成记录（M0 关闭时写第一份）
- 无计划外的应用代码
