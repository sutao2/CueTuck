# 完成记录：M1 桌面骨架

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-22 |
| 里程碑 | M1 |
| 计划 | [2026-08-22-m1-desktop-skeleton.md](../2026-08-22-m1-desktop-skeleton.md) |

## 退出标准

- [x] `cd desktop && npm test`（5 通过）
- [x] `cd desktop/src-tauri && cargo test --locked`（3 通过）
- [x] 主窗口四段壳层对齐原型节奏（顶栏 / 侧栏树 / 内容头与筛选 / 青绿底栏）
- [x] 顶栏或底栏可显示独立 `launcher` 窗口（全局快捷键改到 M3）
- [x] SQLite 可初始化，底栏能表示就绪或未接入
- [x] 工作台「打开应用」「第一期点击广场」「SQLite 就绪」有测试映射
- [x] `./scripts/docs-check` 通过
- [x] 本完成记录已写，status 将 M1 标为完成

## 命令与结果

```text
cd desktop && npm test
Test Files  3 passed (3)
Tests  5 passed (5)

cd desktop/src-tauri && cargo test --locked
test commands::launcher::tests::launcher_label_is_stable ... ok
test local_database::tests::status_is_ready_after_initialize ... ok
test local_database::tests::empty_library_counts_zero ... ok

./scripts/docs-check
docs-check 通过（72 个 Markdown 文件）。
```

目视：工作台壳与原型节奏对齐；广场不发网络请求；浏览器底栏为「SQLite 未接入」属预期。

## 文档

- 更新的规格：workbench、launcher；ADR 0007
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 提示词 CRUD、真实分类数据、合集、使用向导、设置
- 全局快捷键注册与冲突提示（M3 / 设置规格）
