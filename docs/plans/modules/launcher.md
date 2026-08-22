# 模块：启动器

| 字段 | 值 |
|---|---|
| 里程碑 | M1 只做窗口唤起；M3 对齐行为 |
| 规格 | [launcher](../../specs/launcher/spec.md) |
| 源 | [旧启动器来源](../../explanation/legacy-launcher-source.md) |

## 完成时必须为真（M3）

- 独立窗口，不是覆盖层
- 本地搜索不等待网络，1 万条 < 50ms
- 粘贴失败明确降级
- 非 macOS 无选中文本入口

## 依赖

M1 窗口与快捷键；M2 本地库与变量解析。
