# Launcher Palette Implementation Plan

> **For agentic workers:** 按任务执行。不要接广场。启动器只移植旧独立窗口做法，不改成覆盖层。

**Goal:** 启动器整体结构跟旧 `prompt-launcher` 独立窗口：空查询收成搜索栏、有结果后增高、填写态分栏；颜色与字跟当前工作台 token。

**Architecture:** 窗口仍是 label `launcher`、无框透明。高度 `collapsed=80 / expanded=500 / fill=520`。不搬旧广场命令、i18n 和红标。

## Global Constraints

- 空查询不展示结果列表。
- 第一期不出现「去广场」。
- 业务仍走本仓库 SQLite 命令。

---

### Task 1: 窗口高度与结构

- [ ] 测试：`launcherHeightFor` 三种布局；空查询 `is-collapsed`；有结果见页脚键位
- [ ] 实现旧三段结构 + `resizeLauncherWindow`
- [ ] 规格写明调色板做法；`npm test`；`docs-check`
