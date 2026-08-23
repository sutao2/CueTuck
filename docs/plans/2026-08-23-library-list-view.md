# Library List View Implementation Plan

> **For agentic workers:** 不要接广场。只做内容区网格/列表。

**Goal:** 同一批本地结果可在卡片网格和行列表之间切换。

**Architecture:** 已有 `view` 与 `list-view` 类，但列表仍是竖着排的卡片。列表态改成单行：徽章、标题、摘要、操作。

---

- [ ] 失败测试：有提示词时点「列表视图」，容器 `data-layout="list"`，卡片带 `as-row`
- [ ] 最小 CSS / 标记
- [ ] 规格映射；`npm test`；`docs-check`
