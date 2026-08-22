# 完成记录：M0 文档先行

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-22 |
| 里程碑 | M0 |
| 计划 | [milestones/m0.md](../milestones/m0.md) |

## 退出标准

- [x] INDEX 覆盖全部 Markdown
- [x] `./scripts/docs-check` 通过（70 个文件）
- [x] 第一期能力均有 spec
- [x] ADR 0001–0006 存在
- [x] 无应用 `src/` / `desktop/`
- [x] 体系、计划、CI、Cursor 规则随本提交入库
- [x] 本完成记录已写
- [x] status.md 将 M0 标为完成

## 命令与结果

```text
python3 scripts/docs-check
docs-check 通过（70 个 Markdown 文件）。
```

入库后若新增本文件，提交前会再跑一次 docs-check，数字可能为 71。

## 文档

- 更新的规格：documentation（测试映射与 CI/规则）
- 更新的 INDEX：是
- status.md 已改为 M0 完成：是

## 未做 / 下一里程碑带走

- 应用代码
- M1 按 [2026-08-22-m1-desktop-skeleton.md](../2026-08-22-m1-desktop-skeleton.md) 在独立分支执行
