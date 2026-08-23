# User Categories Implementation Plan

> **For agentic workers:** 不要接广场。只做两级分类：大分类下新增小分类，拒绝第三级。

**Goal:** 用户能在系统大分类下新增 `is_system=false` 的小分类；小分类下再创建必须失败。

**Architecture:** 写入走 `create_category_in_dir`。父级必须是 `parent_id IS NULL`。侧栏 ＋ 对当前选中大分类打开输入；选中小分类时只报错。

---

- [x] Rust 失败测试：`creates_user_child_under_office`、`rejects_grandchild_under_frontend`
- [x] JS / 工作台失败测试：内存库同样规则；侧栏能加「周报」、在「前端工程」下失败
- [x] 最小实现 + 规格映射
- [x] `npm test`；`cargo test --locked`；`docs-check`
