# 如何更新文档

## 改现行行为

1. 在 `docs/changes/<短名>/` 复制模板，写提案与规格增量。
2. 实现（若已进入编码里程碑）或先只改文档。
3. 把增量合并进 `docs/specs/<能力>/spec.md`。
4. 更新 [INDEX.md](../INDEX.md)（若新增文件）。
5. 运行 `./scripts/docs-check`。
6. 将变更目录移到 `docs/changes/archive/`，并更新归档说明。

## 改原则或架构

1. 新增 ADR，旧 ADR 标 `superseded` 并链接新篇。
2. 更新宪法或架构文中的现行段落。
3. 跑 docs-check。

## 只改错字

直接改现行文件，不新建 ADR。

## 给 Agent

先读 INDEX，只打开将要改的那一到两份文件。不要把整份 docs 贴进上下文。
