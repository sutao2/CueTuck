# 完成记录：M2 本地工作台

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 里程碑 | M2 |
| 计划 | [2026-08-22-m2-local-workbench.md](../2026-08-22-m2-local-workbench.md) |

## 退出标准

- [x] library / categories / collections / variables / settings 第一期 MUST 有测试映射或声明未开始
- [x] 广场入口仍不发网络请求（`WorkbenchShell.spec.js` keeps square offline）
- [x] `cd desktop && npm test`（16 通过）
- [x] `cd desktop/src-tauri && cargo test --locked`（15 通过）
- [x] `./scripts/docs-check` 通过
- [x] 本完成记录已写，status 将 M2 标为完成

## 命令与结果

```text
cd desktop && npm test
Test Files  5 passed (5)
Tests  16 passed (16)

cd desktop/src-tauri && cargo test --locked
15 passed; 0 failed
（含 creates_empty_collection、adds_member_via_collection_id、
theme_persists_as_dark、import_preview_does_not_write）

./scripts/docs-check
docs-check 通过（75 个 Markdown 文件）。
```

浏览器目视（内存预览）：新建提示词与合集、合集详情加入成员、九宫格缺图仍打开、设置同步页不请求网络、导入先预览、主题切深色。广场仍显示尚未接入。

## 文档

- 更新的规格：library、categories、collections、variables、settings；data-model 记下 `collections` 初始化
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 全局快捷键注册与冲突提示（设置规格「保存快捷键」；M3）
- 启动器接本地搜索与共用 `renderPrompt`（variables「同源渲染」）
- 合集真封面资源；用户新建小分类；拒绝第三级分类的自动化
- 不要在无 M3 计划时整份移植旧启动器
