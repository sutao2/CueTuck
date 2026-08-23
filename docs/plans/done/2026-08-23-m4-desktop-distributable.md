# 完成记录：M4 桌面可分发

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 里程碑 | M4 |
| 计划 | [2026-08-23-m4-desktop-distributable.md](../2026-08-23-m4-desktop-distributable.md) |

## 退出标准

- [x] how-to/local-dev 写出真实命令（不再是「应用尚未开始」）
- [x] 备份/恢复场景有测试或 QA 记录
- [x] 规格无「已实现却未映射」的 MUST
- [x] 不宣传未验证平台（只写 macOS）
- [x] done 记录 + status 更新

## 命令与结果

```text
cd desktop && npm test
Test Files  10 passed (10)
Tests  26 passed (26)

cd desktop/src-tauri && cargo test --locked
21 passed; 0 failed; 1 ignored
（ignored = search_ten_thousand_prompts_bench）
新增：restore_replaces_library、failed_restore_leaves_library

./scripts/docs-check
docs-check 通过（80 个 Markdown 文件）。
```

手工 QA：[how-to/release-qa.md](../../how-to/release-qa.md)。浏览器走通新建、筛选、导入预览、库文件备份拒绝、广场离线、独立启动器页。库文件替换/回滚以 Rust 单测为准。未打桌面发行标签，无商店包。

## 文档

- 更新的规格：settings（备份恢复场景 + 映射）
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 合集真封面；用户新建小分类；拒绝第三级的自动化；网格/列表切换
- Playwright CI（test-gates 标未启用）
- 签名、商店上架、Windows / Linux 安装包
- 不要在无 M5 计划时接广场或后端
