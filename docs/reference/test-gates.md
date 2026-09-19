# 测试门禁

| 字段 | 值 |
|---|---|
| 状态 | 现行 |
| 关联 | [质量](quality.md) · [路线图](../product/roadmap.md) |

下表列出各阶段要求；实际自动执行入口为 `scripts/verify.mjs` 和 `.github/workflows/verify.yml`。尚未落实的要求单独标出，不得将普通测试通过写成全部门禁达标。

## 本地提交策略

按用户 2026-09-08 的明确决定，本仓库不启用 WorkLog IDE 评审批准作为 Git 提交前置条件，不得自动重新安装 `WORKLOG_REVIEW_GATE`。本机原钩子已改名保留备份，不使用单次跳过参数或伪造评审结果。测试、文档检查、密钥保护与 CI 要求继续有效。操作范围见[关闭计划](../plans/2026-09-08-disable-worklog-gate.md)。

## 始终启用（M0 起）

| 门 | 命令或动作 | 失败则 |
|---|---|---|
| 文档结构 | `./scripts/docs-check`（本地与 `.github/workflows/docs.yml`） | 不得合并 |
| API 路由合同 | `node scripts/api-contract-check.mjs`（集中 frontend 验证及 full-regression CI） | 后端路径/显式 HTTP 方法与两个 OpenAPI 双向不一致，不得合并 |
| 密钥 | 不把 `.env`、钥匙、证书提交进库 | 不得合并 |

## M1 起

| 门 | 范围 | 失败则 |
|---|---|---|
| 前端单测 | 数据访问、窗口适配、纯函数 | 不得合并 |
| Rust 检查 | `cargo test --locked`（启动器/库初始化相关） | 不得合并 |

## M2 起

| 门 | 范围 | 失败则 |
|---|---|---|
| 组件测试 | 工作台、分类树、卡片、使用向导 | 不得合并 |
| 规格映射 | 每个已实现 MUST 能指到测试名 | 不得合并该能力 |

## M3 起

| 门 | 范围 | 失败则 |
|---|---|---|
| 启动器测试 | 搜索、键盘、变量渲染、粘贴降级 | 不得合并 |
| 本地查询基准 | 1 万条 < 50ms | 不得合并 |

## 核心流程快速回归

`npm --prefix desktop run test:core` 集中运行登录恢复、启动器搜索/复制/变量/AI 入口、翻译鉴权、公开附件、下载计数、更新状态与推荐分页测试。失败须修复后再交付；不替代全量 `verify.mjs`、原生系统剪贴板与跨平台安装验收。这些文件已经属于全量 CI 测试集合，无须单独重复运行 CI 任务。启动器排序与新增映射见[本轮计划](../plans/2026-09-15-launcher-ranking.md)。

## M4 起

| 门 | 范围 | 失败则 |
|---|---|---|
| 本机 smoke | [发行前 QA](../how-to/release-qa.md) + 备份恢复单测；或 `tauri dev` 走同一清单 | 不得打桌面标签 |
| E2E | `node scripts/verify.mjs browser`；Playwright CLI 隔离新建/编辑/变量复制/删除确认/设置返回 | 已接入 `full-regression`；本机结果见 [收尾记录](../plans/2026-09-08-local-release-readiness.md)，远端未跑不得宣称通过 |

## M5 起

| 门 | 范围 | 失败则 |
|---|---|---|
| 后端测试（已接入） | `cargo test --locked -- --test-threads=4`；CI 使用 PostgreSQL service，测试使用隔离 schema | 不得合并后端 |
| 后端覆盖率与容器要求（尚未落实） | [ADR 0008](../architecture/decisions/0008-m5-backend-contract.md) 要求新后端生产代码行覆盖率 ≥ 80%、分支覆盖率 ≥ 70%，API 用 Testcontainers；当前没有覆盖率采集/阈值阻断，也没有 Testcontainers 集成 | 不得合并后端的要求仍有效，但尚未自动阻断；不得宣称已达标，普通 CI 通过不解除此缺口 |
| 合同（已接入） | 真实路由与 OpenAPI 路径/方法双向检查；客户端合同测试核对文档中的认证标记；行为由对应后端测试验证 | 不得合并 API 变更 |

路由检查支持当前 `backend/src/lib.rs` 中的字面路径与内联 MethodRouter；参数名如 `:flow_id` 和 `{flowId}` 归一化比较，检查显式注册的方法，不把 Axum 隐式 HEAD 或中间件 OPTIONS 算作新增接口。新增路径/方法、残留文档或不支持的路由注册形式须报错；采用嵌套路由等新形式时先扩展检查。它不证明请求/响应字段和运行时权限正确，OpenAPI 说明仍需对照处理器审核。失败场景回归见 `scripts/api-contract-check.test.mjs`。

## M8 起

管理端浏览器回归同样由 `node scripts/verify.mjs browser` 执行；隔离 API 替身与真实 PostgreSQL 测试分工、证据及外部验证边界见[管理端补齐验收](../plans/2026-09-08-admin-hardening.md)。

| 门 | 范围 | 失败则 |
|---|---|---|
| 设置对齐 | 十类导航与已接通本机行有规格映射；已有 JSON/备份/主题/唤起快捷键测试仍绿 | 不得合并该能力 |

## M9 起

| 门 | 范围 | 失败则 |
|---|---|---|
| 本机 MCP | `mcp` 本地搜索 / 读取 / 渲染不联网；显式启用的独立广场工具按 [ADR 0021](../architecture/decisions/0021-mcp-square-opt-in.md) 验证 | 不得合并该能力 |
| 浏览器工作台 | 独立 `web/` 不进桌面包；窄桌面侧栏可收起 | 不得合并该能力 |

## 规格到测试

集中入口及各端 CI 运行说明以 [部署与验收模板](../../deploy/README.md) 为准。本机独立合成数据库、附件恢复演练不等于正式灾备；签名与其他 OS 实机仍是发行门槛。

实现某能力时，在该 `spec.md` 文末增加「测试映射」表：`场景 → 测试文件::名称`。未实现的场景写 `未开始`，不得删场景来躲门禁。

## 手工 QA（发行前）

仅当无法稳定自动化时写入本表：

| 项 | 适用 | 记录位置 |
|---|---|---|
| 全局快捷键与系统冲突 | 各操作系统 | 发行笔记 |
| 辅助功能权限拒绝后的粘贴降级 | macOS | 发行笔记 |
