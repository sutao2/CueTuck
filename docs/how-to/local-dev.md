# 如何在本机工作

文档与桌面前端探活已可用。Tauri 窗口与 SQLite 仍在 M1 后续任务。

## 检查文档

```bash
./scripts/docs-check
```

必须退出码 0。

## 桌面前端

需要 Node.js 22+。

```bash
cd desktop
npm install
npm test
npm run dev
```

## 阅读顺序（新人）

1. [README](../../README.md) 与 [文档体系](../README.md)
2. [宪法](../constitution.md)
3. [PRD](../product/prd.md)
4. [路线图](../product/roadmap.md)
5. 按任务打开一份 `docs/specs/<能力>/spec.md`

Tauri 与 SQLite 命令在 M1 Task 2–4 完成后再写入本页。
