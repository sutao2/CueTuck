# 6. 计划只写一层深

- Status: accepted
- Date: 2026-08-22
- 关联：[程序计划](../../plans/program.md) · [路线图](../../product/roadmap.md)

## Context

把 M1–M6 的逐步任务和「已经完成」的状态一次写完，看起来完整，但 SDD 实践（Spec Kit / OpenSpec / 2026 年对瀑布回潮的批评）表明：详细任务超过一个里程碑就会在实现后失效，Agent 会执行过时步骤。

## Decision

- 永久保存：程序计划、模块地图、每个里程碑的进入/退出标准和完成定义。
- 开工前才写：当前里程碑的逐步实现计划（`docs/plans/YYYY-MM-DD-<模块>.md`）。
- 完成后才写：`docs/plans/done/` 里的完成记录，必须附命令与证据。禁止预填「已完成」。

## Consequences

- M2 及以后没有假文件路径、假测试名。
- 状态以 [status.md](../../plans/status.md) 的当前事实为准。
- 关闭一个里程碑后，再写下一份逐步计划。
