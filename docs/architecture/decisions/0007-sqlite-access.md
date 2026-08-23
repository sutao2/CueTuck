# 7. 本地库走 Rust rusqlite

- Status: accepted
- Date: 2026-08-22
- 关联：[数据模型](../data-model.md)

## Context

M1 需要可单测的 SQLite 初始化。`tauri-plugin-sql` 便于前端直接查库，但初始化与状态机更难在无窗口时测。

## Decision

用 Rust 侧 `rusqlite`（bundled）打开 `promptark.sqlite`。前端只调用 `initialize_local_database`、`get_local_database_status`、`count_local_prompts`。

## Consequences

- `initialize_in_dir` 可在 `cargo test` 里用临时目录验证。
- 业务 CRUD 也走命令层，不把 SQL 散落到 Vue。
- 未采用 tauri-plugin-sql。
