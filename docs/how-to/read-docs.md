# 如何按需读文档

为省 token：先索引，再最多打开两份现行文件。

## 固定开场

1. [AGENTS.md](../../AGENTS.md) 或 [CLAUDE.md](../../CLAUDE.md)
2. [INDEX.md](../INDEX.md)

不要把 `docs/` 整树读进上下文。

## 按问题选文件

| 问题 | 打开 |
|---|---|
| 能不能做、做多大 | 宪法 + PRD |
| 做到哪一阶段 | 路线图 |
| 这个词什么意思 | 术语 |
| 这个功能必须怎样 | `specs/<能力>/spec.md` |
| 表怎么设计 | 数据模型 |
| 为什么选这个方案 | 对应 ADR |
| 测试要过什么 | 测试门禁 |
| 主窗口长什么样 | 工作台规格 + 设计源说明 |
| 启动器复制哪些代码 | 启动器规格 + 旧启动器来源 |
| 现在做到哪 | [status.md](../plans/status.md) |
| 下一期怎么一步步做 | 当前的 `docs/plans/YYYY-MM-DD-*.md` |
| 某能力做完长什么样 | `docs/plans/modules/<模块>.md` |
| 管理台能不能做、合同在哪 | [admin 规格](../specs/admin/spec.md) + [ADR 0009](../architecture/decisions/0009-m6-admin-console.md) |
| 收藏写路径、令牌轮换、admin me | [M7 提案](../changes/m7-contract-gaps/proposal.md) + [ADR 0010](../architecture/decisions/0010-m7-contract-gaps.md) |
| 原型设置项为什么缺、何时补 | [设置规格](../specs/settings/spec.md) + [M8 完成记录](../plans/done/2026-08-23-m8-settings-ia.md) |
| 浏览器工作台或 MCP 怎么做 | [ADR 0011](../architecture/decisions/0011-web-and-mcp.md) + [M9 计划](../plans/2026-08-24-m9-web-and-mcp.md) |

## 进行中的工作

`docs/changes/<改动>/` 只在任务明确说「做这一份变更」时读。默认以 `docs/specs/` 为真相。
