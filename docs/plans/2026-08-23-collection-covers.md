# Collection Covers Implementation Plan

> **For agentic workers:** 不要接广场。只把合集封面写成 `cover_json` 资源引用并显示出来。

**Goal:** 合集能保存单图或九宫格封面引用；详情按顺序出图，缺图用占位且仍能打开；内容区卡片能看到已有图。

**Architecture:** `cover_json` 是字符串数组。创建时写入。详情最多 9 格，卡片取前 3 张。本轮不新建 `assets` 表。

---

- [x] Rust：`persists_grid_cover_refs`（三张引用写入后能读回）
- [x] JS：内存库写入 `cover_json`；详情 3 图 + 6 占位；卡片预览
- [x] 创建弹窗可选图；规格映射
- [x] `npm test`；`cargo test --locked`；`docs-check`
