# Agent 入口

先读本文件和 [docs/INDEX.md](docs/INDEX.md)。不要通读 `docs/`。按问题选文件见 [如何按需读文档](docs/how-to/read-docs.md)。

## 强制顺序

1. 打开 INDEX，按「何时读」选文件。
2. 一次最多打开 2 份现行文档，再动手。
3. 改结构、行为或门禁时，同一提交更新对应规格或 ADR，并更新 INDEX。
4. 没进 INDEX 的文档视为不存在。
5. 未完成 `docs/plans/` 中该模块计划前，不写应用代码。
6. 启动器只移植旧仓库独立窗口实现，不改成原型覆盖层。

## 现行真相

- 宪法：[docs/constitution.md](docs/constitution.md)
- 需求：[docs/product/prd.md](docs/product/prd.md)
- 能力规格：`docs/specs/<能力>/spec.md`
- 进行中的变更：`docs/changes/<改动>/`（未归档的不是现行真相）

## 禁止

- 把同一事实复制到第二份文件
- 静默改写已接受的 ADR；过时则标 `superseded` 并另写新篇
- 为了省事跳过 Given/When/Then 场景
- 读取 `../PromptLauncher` 除启动器相关代码与「提示词软件 2」原型以外的实现当现行合同
