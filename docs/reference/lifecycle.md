# 文档生命周期

| 字段 | 值 |
|---|---|
| 状态 | 现行 |
| 关联 | [文档体系](../README.md) · [如何更新文档](../how-to/update-docs.md) |

## 新能力

1. 在 PRD / 路线图确认该能力属于当前里程碑。
2. 用 [规格模板](../templates/capability-spec.md) 写 `docs/specs/<能力>/spec.md`。
3. 把文件加入 [INDEX.md](../INDEX.md)。
4. 写 `docs/plans/YYYY-MM-DD-<能力>.md`。
5. 按计划实现，场景补进规格的「测试映射」。
6. 跑 `./scripts/docs-check` 与该里程碑的测试门禁。

## 改已有能力

1. 复制 `docs/changes/_template/` 为 `docs/changes/<短名>/`。
2. 写提案、设计、任务和规格增量。
3. 实现后把增量合并进现行 `spec.md`。
4. 把变更目录说明移到 [archive](../changes/archive/README.md)。

## 改决定

1. 新 ADR；旧篇 `Status: superseded` 并链接新篇。
2. 更新宪法或架构里的现行段落。
3. 更新 INDEX（若新文件）。

## 禁止

- 先改代码再补规格
- 在两份文件写同一条 MUST
- 删除场景来让测试变绿
