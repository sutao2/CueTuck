# 文档体系

本仓库用文档驱动开发。人从本页理解体系，查具体文件走 [INDEX.md](INDEX.md)。

## 一条链

```text
宪法 → 产品需求 / 路线图
     → 能力规格（现行合同）
     → 变更提案（未合并的增量）
     → 实现计划
     → 代码与测试
     → 合并回规格，变更归档
```

跳过计划写代码，或规格与代码各写各的，都算破坏体系。

## 四类内容

| 类型 | 目录 | 作用 |
|---|---|---|
| 解释与原则 | `constitution.md`、`explanation/` | 为什么 |
| 需求与规格 | `product/`、`specs/` | 必须怎样表现 |
| 参考 | `architecture/`、`reference/`、ADR | 结构、门禁、已做决定 |
| 操作 | `how-to/`、`tutorials/`、`plans/` | 怎么做 |

## 谁维护什么

| 角色 | 只改这些 |
|---|---|
| 定范围 | 宪法、PRD、路线图、术语 |
| 定行为 | `specs/<能力>/spec.md` |
| 定结构 | 架构、数据模型、ADR |
| 开工 | `changes/` 然后 `plans/` |
| 日常查 | INDEX，不要通读本目录 |

## 硬门禁

本地与 CI 都跑 `./scripts/docs-check`：孤儿文档、INDEX 空链、相对断链、ADR 缺段。步骤见 [如何更新文档](how-to/update-docs.md)。
