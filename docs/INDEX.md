# 文档索引

Agent 只读本表，再打开需要的文件。没出现在本表的文档视为不存在。

图例：`现行` = 今天为真；`目标` = 已指定尚未实现；`模板` = 复制用；`归档` = 非现行真相。

| 路径 | 状态 | 何时读 | 一句话 |
|---|---|---|---|
| [../README.md](../README.md) | 现行 | 人第一次进仓库 | 产品一句话与入口 |
| [../backend/README.md](../backend/README.md) | 现行 | 跑 M5 API | 本仓库会话服务 |
| [../admin-web/README.md](../admin-web/README.md) | 现行 | 跑管理台预览 | 独立 admin-web；不进桌面包 |
| [../web/README.md](../web/README.md) | 现行 | 跑浏览器工作台 | 独立 web；不进桌面包 |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | 现行 | 准备提交改动 | 参与规则 |
| [../AGENTS.md](../AGENTS.md) | 现行 | 任何 Agent 开场 | 阅读顺序与禁令 |
| [../CLAUDE.md](../CLAUDE.md) | 现行 | Claude/Cursor 开场 | 指向 INDEX 的薄入口 |
| [../.github/PULL_REQUEST_TEMPLATE.md](../.github/PULL_REQUEST_TEMPLATE.md) | 现行 | 开 PR | PR 文档检查清单 |
| [README.md](README.md) | 现行 | 问文档体系怎么运转 | 体系总说明 |
| [INDEX.md](INDEX.md) | 现行 | 每次查文档 | 本表 |
| [constitution.md](constitution.md) | 现行 | 改原则或开新模块前 | 非协商约束 |
| [product/prd.md](product/prd.md) | 现行 | 问范围、做什么、不做什么 | 完整产品需求 |
| [product/roadmap.md](product/roadmap.md) | 现行 | 问进度或下一步里程碑 | 里程碑与完成标准 |
| [product/glossary.md](product/glossary.md) | 现行 | 用词含糊时 | 术语唯一定义 |
| [architecture/overview.md](architecture/overview.md) | 现行 | 问系统怎么拆 | 容器与窗口 |
| [architecture/data-model.md](architecture/data-model.md) | 现行 | 改表或字段 | 本地 SQLite 目标模型 |
| [architecture/decisions/0001-greenfield-sibling-repo.md](architecture/decisions/0001-greenfield-sibling-repo.md) | 现行 | 问为什么不在旧仓库改 | 独立仓库 |
| [architecture/decisions/0002-preserve-current-launcher.md](architecture/decisions/0002-preserve-current-launcher.md) | 现行 | 动启动器前 | 保留旧启动器 |
| [architecture/decisions/0003-local-first-phase1.md](architecture/decisions/0003-local-first-phase1.md) | 现行 | 想接后端时 | 第一期纯本地 |
| [architecture/decisions/0004-documentation-system.md](architecture/decisions/0004-documentation-system.md) | 现行 | 改文档规则时 | 文档组合方案 |
| [architecture/decisions/0005-ui-source-prompt-ark-prototype.md](architecture/decisions/0005-ui-source-prompt-ark-prototype.md) | 现行 | 改主窗口视觉时 | 原型是设计源 |
| [architecture/decisions/0006-plan-altitude.md](architecture/decisions/0006-plan-altitude.md) | 现行 | 想一次写完所有逐步任务时 | 计划只写一层深 |
| [architecture/decisions/0007-sqlite-access.md](architecture/decisions/0007-sqlite-access.md) | 现行 | 改本地库访问方式时 | rusqlite 而不是 plugin-sql |
| [architecture/decisions/0008-m5-backend-contract.md](architecture/decisions/0008-m5-backend-contract.md) | 现行 | 接广场或后端前 | 改写 API；邮箱密码；覆盖率 |
| [architecture/decisions/0009-m6-admin-console.md](architecture/decisions/0009-m6-admin-console.md) | 现行 | 做管理台或审核 API 前 | 独立 admin.yaml；admin-web 不进桌面 |
| [architecture/decisions/0010-m7-contract-gaps.md](architecture/decisions/0010-m7-contract-gaps.md) | 现行 | 做已登录收藏、轮换或 admin me 前 | 收藏是账号关系；Refresh 轮换；GET /v1/admin/me |
| [architecture/decisions/0011-web-and-mcp.md](architecture/decisions/0011-web-and-mcp.md) | 现行 | 做浏览器工作台或 MCP 前 | Web 独立 SPA；Agent 入口是 MCP stdio |
| [architecture/decisions/0012-postgres-backend.md](architecture/decisions/0012-postgres-backend.md) | 现行 | 接 Postgres、Redis、MinIO 或改口令存储时 | 预发存本机 `promptark` 库；Argon2id |
| [architecture/decisions/0013-oauth-google-github.md](architecture/decisions/0013-oauth-google-github.md) | 现行 | 接 Google / GitHub 登录前 | 选定 Google 与 GitHub；仍不接 QQ/LinuxDo |
| [architecture/decisions/0014-full-product.md](architecture/decisions/0014-full-product.md) | 现行 | 问是否还按第一期冻结 | 剩余工作按完整产品排队 |
| [specs/launcher/spec.md](specs/launcher/spec.md) | 目标 | 做启动器 | 独立窗口行为合同 |
| [specs/workbench/spec.md](specs/workbench/spec.md) | 目标 | 做主窗口壳 | 工作台壳层 |
| [specs/library/spec.md](specs/library/spec.md) | 目标 | 做本地 CRUD | 本地提示词 |
| [specs/square/spec.md](specs/square/spec.md) | 目标 | 改广场浏览、下载或收藏 | 广场；M5 已实现浏览与匿名下载 |
| [specs/collections/spec.md](specs/collections/spec.md) | 目标 | 做合集 | 合集 |
| [specs/categories/spec.md](specs/categories/spec.md) | 目标 | 做分类树 | 两级分类 |
| [specs/variables/spec.md](specs/variables/spec.md) | 目标 | 做使用向导或渲染 | 变量解析 |
| [specs/auth/spec.md](specs/auth/spec.md) | 目标 | 改登录或令牌存放 | 认证；邮箱密码；Refresh 进钥匙串 |
| [specs/settings/spec.md](specs/settings/spec.md) | 目标 | 做设置 | 本机设置；M8 已对齐原型十类 |
| [specs/publish/spec.md](specs/publish/spec.md) | 目标 | 改发布提交 | 发布；选源后提交，不锁本地 |
| [specs/admin/spec.md](specs/admin/spec.md) | 目标 | 做管理台或审核写路径 | 独立管理端；不进桌面包 |
| [specs/web/spec.md](specs/web/spec.md) | 目标 | 做浏览器工作台 | 流体桌面布局；不进桌面包 |
| [specs/mcp/spec.md](specs/mcp/spec.md) | 目标 | 做本机 MCP | stdio 查询本地提示词 |
| [specs/sync/spec.md](specs/sync/spec.md) | 目标 | 做个人库云同步 | 登录后推拉账号库 |
| [specs/billing/spec.md](specs/billing/spec.md) | 目标 | 做预发账单或兑换 | 不得把未付费写成 Pro |
| [specs/documentation/spec.md](specs/documentation/spec.md) | 现行 | 改 docs-check 或索引规则 | 文档门禁合同 |
| [reference/test-gates.md](reference/test-gates.md) | 现行 | 加测试或 CI | 分阶段门禁 |
| [reference/openapi/square.yaml](reference/openapi/square.yaml) | 现行 | 改广场 API 时 | M5 广场 / 登录 / 发布合同 |
| [reference/openapi/admin.yaml](reference/openapi/admin.yaml) | 现行 | 改管理 API 时 | M6 管理合同；/v1/admin |
| [reference/quality.md](reference/quality.md) | 现行 | 评审标准含糊时 | 质量约定 |
| [how-to/local-dev.md](how-to/local-dev.md) | 现行 | 想在本机验证 | npm test / npm run dev / tauri dev / 备份恢复 |
| [how-to/release-qa.md](how-to/release-qa.md) | 现行 | 发行前手工 smoke | M4 QA 表；未验证平台不得勾选 |
| [how-to/read-docs.md](how-to/read-docs.md) | 现行 | Agent 或人要省 token | 按问题打开哪份 |
| [how-to/update-docs.md](how-to/update-docs.md) | 现行 | 要改规格或 ADR | 文档更新步骤 |
| [reference/lifecycle.md](reference/lifecycle.md) | 现行 | 问文档怎么流转 | 从规格到归档 |
| [tutorials/first-run.md](tutorials/first-run.md) | 现行 | 第一次读仓库 | 十分钟走完 M0 |
| [explanation/ui-source.md](explanation/ui-source.md) | 现行 | 问主窗口为什么换皮 | 设计源说明 |
| [explanation/legacy-launcher-source.md](explanation/legacy-launcher-source.md) | 现行 | 问启动器复制哪些文件 | 旧启动器范围 |
| [changes/README.md](changes/README.md) | 现行 | 开新变更前 | 变更目录怎么用 |
| [changes/archive/README.md](changes/archive/README.md) | 现行 | 合并变更后 | 归档说明 |
| [changes/_template/proposal.md](changes/_template/proposal.md) | 模板 | 开变更 | 提案模板 |
| [changes/_template/design.md](changes/_template/design.md) | 模板 | 开变更 | 设计模板 |
| [changes/_template/tasks.md](changes/_template/tasks.md) | 模板 | 开变更 | 任务模板 |
| [changes/m5-backend-contract/proposal.md](changes/m5-backend-contract/proposal.md) | 现行 | 查 M5 合同是否已接受 | 已接受；决定以 ADR 0008 为准 |
| [changes/m5-backend-contract/design.md](changes/m5-backend-contract/design.md) | 目标 | 看 M5 合同怎么落地 | OpenAPI、Rust 持令牌、本仓库后端 |
| [changes/m6-admin-console/proposal.md](changes/m6-admin-console/proposal.md) | 现行 | 查 M6 合同是否已接受 | 已接受；决定以 ADR 0009 为准 |
| [changes/m6-admin-console/design.md](changes/m6-admin-console/design.md) | 目标 | 看 M6 管理台怎么落地 | admin.yaml、同一 backend、独立 admin-web |
| [changes/m7-contract-gaps/proposal.md](changes/m7-contract-gaps/proposal.md) | 现行 | 查 M7 合同是否已接受 | 已接受；决定以 ADR 0010 为准 |
| [changes/m7-contract-gaps/design.md](changes/m7-contract-gaps/design.md) | 目标 | 看 M7 怎么落地 | 收藏表、令牌轮换、GET /v1/admin/me |
| [changes/m8-settings-ia/proposal.md](changes/m8-settings-ia/proposal.md) | 现行 | 查设置对齐是否已写入合同 | 已接受；只增不减；M7 已关闭可改设置代码 |
| [changes/m8-settings-ia/design.md](changes/m8-settings-ia/design.md) | 目标 | 看 M8 设置怎么落地 | 十类导航、本机行、云行诚实占位 |
| [changes/m9-web-and-mcp/proposal.md](changes/m9-web-and-mcp/proposal.md) | 现行 | 查 M9 合同是否已接受 | 已接受；决定以 ADR 0011 为准 |
| [changes/m9-web-and-mcp/design.md](changes/m9-web-and-mcp/design.md) | 目标 | 看 M9 怎么落地 | 独立 web/；mcp stdio 读 SQLite |
| [changes/postgres-backend/proposal.md](changes/postgres-backend/proposal.md) | 现行 | 查预发后端持久化是否已接受 | 已接受；ADR 0012 / 0013 |
| [changes/postgres-backend/design.md](changes/postgres-backend/design.md) | 目标 | 看预发后端怎么接到本机库 | 独立库 promptark；Argon2；OAuth；MinIO |
| [changes/oauth-clients/proposal.md](changes/oauth-clients/proposal.md) | 现行 | 查客户端 OAuth 是否已接受 | 已接受；登录弹窗接 Google / GitHub |
| [changes/full-product/proposal.md](changes/full-product/proposal.md) | 现行 | 查完整产品队列是否已接受 | 已接受；ADR 0014 |
| [changes/full-product/design.md](changes/full-product/design.md) | 目标 | 看同步 / 更新 / 账单怎么落地 | 账号库推拉；updater；预发兑换 |
| [plans/README.md](plans/README.md) | 现行 | 准备写或找计划 | 计划目录规则 |
| [plans/program.md](plans/program.md) | 现行 | 问总顺序和依赖 | 程序计划 |
| [plans/status.md](plans/status.md) | 现行 | 问现在做到哪 | 只写今天为真的状态 |
| [plans/2026-08-22-m1-desktop-skeleton.md](plans/2026-08-22-m1-desktop-skeleton.md) | 归档 | 查 M1 怎么做的 | M1 逐步实现计划 |
| [plans/2026-08-22-m2-local-workbench.md](plans/2026-08-22-m2-local-workbench.md) | 归档 | 查 M2 怎么做的 | M2 逐步实现计划 |
| [plans/2026-08-23-m3-launcher.md](plans/2026-08-23-m3-launcher.md) | 归档 | 查 M3 怎么做的 | M3 逐步实现计划 |
| [plans/2026-08-23-m4-desktop-distributable.md](plans/2026-08-23-m4-desktop-distributable.md) | 归档 | 查 M4 怎么做的 | M4 逐步实现计划 |
| [plans/2026-08-23-macos-window-chrome.md](plans/2026-08-23-macos-window-chrome.md) | 归档 | 查 macOS 窗框怎么做的 | 红绿灯 / overlay / ⌃Space |
| [plans/2026-08-23-launcher-palette.md](plans/2026-08-23-launcher-palette.md) | 归档 | 查启动器调色板怎么做的 | 收起 / 展开 / 填写 |
| [plans/2026-08-23-library-list-view.md](plans/2026-08-23-library-list-view.md) | 归档 | 查列表视图怎么做的 | 同一批结果改成行 |
| [plans/2026-08-23-user-categories.md](plans/2026-08-23-user-categories.md) | 归档 | 查用户小分类怎么做的 | 两级上限，可新增小分类 |
| [plans/2026-08-23-collection-covers.md](plans/2026-08-23-collection-covers.md) | 归档 | 查合集真封面怎么做的 | cover_json 引用与缺图占位 |
| [plans/2026-08-23-m5-online-square.md](plans/2026-08-23-m5-online-square.md) | 归档 | 查 M5 怎么做的 | M5 逐步实现计划 |
| [plans/2026-08-23-m6-admin-console.md](plans/2026-08-23-m6-admin-console.md) | 归档 | 查 M6 怎么做的 | M6 逐步实现计划 |
| [plans/2026-08-23-m7-contract-gaps.md](plans/2026-08-23-m7-contract-gaps.md) | 归档 | 查 M7 怎么做的 | M7 逐步实现计划 |
| [plans/2026-08-23-m8-settings-ia.md](plans/2026-08-23-m8-settings-ia.md) | 归档 | 查 M8 怎么做的 | M8 逐步实现计划 |
| [plans/2026-08-24-use-wizard-steps.md](plans/2026-08-24-use-wizard-steps.md) | 归档 | 查使用向导逐步填写怎么测的 | 一次只问一个变量 |
| [plans/2026-08-24-use-wizard-no-vars.md](plans/2026-08-24-use-wizard-no-vars.md) | 归档 | 查无变量预览与 Enter 怎么测的 | 无 `{{` 直接预览；Enter 前进 |
| [plans/2026-08-24-windows-nsis.md](plans/2026-08-24-windows-nsis.md) | 归档 | 查 Windows NSIS 怎么配的 | 不声称 Windows 已验证 |
| [plans/2026-08-24-m9-web-and-mcp.md](plans/2026-08-24-m9-web-and-mcp.md) | 归档 | 查 M9 怎么做的 | 先 MCP 后 Web |
| [plans/2026-08-24-square-preview-gaps.md](plans/2026-08-24-square-preview-gaps.md) | 归档 | 查广场预发缺口怎么补的 | 详情 / 排序 / 审核进列表 |
| [plans/2026-08-24-web-edit-prompt.md](plans/2026-08-24-web-edit-prompt.md) | 归档 | 查浏览器内存库编辑怎么做的 | 浏览器内存库编辑 |
| [plans/2026-08-24-web-use-wizard.md](plans/2026-08-24-web-use-wizard.md) | 归档 | 查浏览器使用向导怎么做的 | 浏览器逐步填写 |
| [plans/2026-08-24-web-square-preview.md](plans/2026-08-24-web-square-preview.md) | 归档 | 查浏览器接预发广场怎么做的 | 浏览器接预发广场 |
| [plans/2026-08-24-postgres-backend.md](plans/2026-08-24-postgres-backend.md) | 归档 | 查预发后端怎么接到 Postgres | Argon2、OAuth API、Redis、MinIO |
| [plans/2026-08-25-oauth-clients.md](plans/2026-08-25-oauth-clients.md) | 归档 | 查客户端 OAuth 怎么接到登录弹窗 | 桌面钥匙串；web/admin 不写 Refresh |
| [plans/2026-08-25-account-surface.md](plans/2026-08-25-account-surface.md) | 归档 | 查作者主页、我的发布、下载作者怎么接到设置 | 账号面剩余行 |
| [plans/2026-08-25-library-sync.md](plans/2026-08-25-library-sync.md) | 归档 | 查个人库云同步怎么做的 | 登录后推拉账号库 |
| [plans/2026-08-25-auto-update.md](plans/2026-08-25-auto-update.md) | 归档 | 查自动更新安装怎么做的 | GitHub Releases；不上架商店 |
| [plans/2026-08-25-win-linux-prefs.md](plans/2026-08-25-win-linux-prefs.md) | 现行 | 做 Windows / Linux 开机启动与托盘 | 未验证不得勾 QA |
| [plans/2026-08-25-preview-billing.md](plans/2026-08-25-preview-billing.md) | 目标 | 做预发账单与兑换 | 无密钥不得写成 Pro |
| [plans/deferred.md](plans/deferred.md) | 目标 | 问商店 / 生产托管 / NSIS 额度 | 没有证据就不能声称 |
| [plans/milestones/m0.md](plans/milestones/m0.md) | 现行 | 关闭或检查 M0 | M0 进出标准 |
| [plans/milestones/m1.md](plans/milestones/m1.md) | 现行 | 做桌面骨架 | M1 进出标准 |
| [plans/milestones/m2.md](plans/milestones/m2.md) | 现行 | 做本地工作台前 | M2 进出标准 |
| [plans/milestones/m3.md](plans/milestones/m3.md) | 现行 | 对齐启动器前 | M3 进出标准 |
| [plans/milestones/m4.md](plans/milestones/m4.md) | 现行 | 准备可分发前 | M4 进出标准 |
| [plans/milestones/m5.md](plans/milestones/m5.md) | 现行 | 接广场前 | M5 进出标准 |
| [plans/milestones/m6.md](plans/milestones/m6.md) | 现行 | 做管理台前 | M6 进出标准 |
| [plans/milestones/m7.md](plans/milestones/m7.md) | 现行 | 做合同补齐前 | M7 进出标准 |
| [plans/milestones/m8.md](plans/milestones/m8.md) | 现行 | 做设置对齐前 | M8 进出标准 |
| [plans/milestones/m9.md](plans/milestones/m9.md) | 现行 | 关闭或检查 M9 | M9 进出标准 |
| [plans/modules/README.md](plans/modules/README.md) | 现行 | 问模块怎么切 | 模块地图 |
| [plans/modules/workbench.md](plans/modules/workbench.md) | 现行 | 做主窗口壳 | 工作台模块完成态 |
| [plans/modules/library.md](plans/modules/library.md) | 现行 | 做本地库 | 本地库模块完成态 |
| [plans/modules/categories.md](plans/modules/categories.md) | 现行 | 做分类 | 分类模块完成态 |
| [plans/modules/collections.md](plans/modules/collections.md) | 现行 | 做合集 | 合集模块完成态 |
| [plans/modules/variables.md](plans/modules/variables.md) | 现行 | 做使用向导 | 变量模块完成态 |
| [plans/modules/settings.md](plans/modules/settings.md) | 现行 | 做设置 | 设置模块完成态（M2 子集 + M8 十类） |
| [plans/modules/launcher.md](plans/modules/launcher.md) | 现行 | 做启动器 | 启动器模块完成态 |
| [plans/modules/square.md](plans/modules/square.md) | 现行 | M5 广场 | 广场模块完成态 |
| [plans/modules/admin.md](plans/modules/admin.md) | 现行 | M6 管理台 | 管理台模块完成态 |
| [plans/modules/web.md](plans/modules/web.md) | 现行 | 做浏览器工作台 | Web 模块完成态 |
| [plans/modules/mcp.md](plans/modules/mcp.md) | 现行 | 做本机 MCP | MCP 模块完成态 |
| [plans/done/README.md](plans/done/README.md) | 现行 | 里程碑做完后 | 完成记录怎么写 |
| [plans/done/_template.md](plans/done/_template.md) | 模板 | 写完成记录 | 完成记录模板 |
| [plans/done/2026-08-22-m0-documentation.md](plans/done/2026-08-22-m0-documentation.md) | 归档 | 查 M0 是否关闭 | M0 完成证据 |
| [plans/done/2026-08-22-m1-desktop-skeleton.md](plans/done/2026-08-22-m1-desktop-skeleton.md) | 归档 | 查 M1 是否关闭 | M1 完成证据 |
| [plans/done/2026-08-23-m2-local-workbench.md](plans/done/2026-08-23-m2-local-workbench.md) | 归档 | 查 M2 是否关闭 | M2 完成证据 |
| [plans/done/2026-08-23-m3-launcher.md](plans/done/2026-08-23-m3-launcher.md) | 归档 | 查 M3 是否关闭 | M3 完成证据 |
| [plans/done/2026-08-23-m4-desktop-distributable.md](plans/done/2026-08-23-m4-desktop-distributable.md) | 归档 | 查 M4 是否关闭 | M4 完成证据 |
| [plans/done/2026-08-23-m5-online-square.md](plans/done/2026-08-23-m5-online-square.md) | 归档 | 查 M5 是否关闭 | M5 完成证据 |
| [plans/done/2026-08-23-m6-admin-console.md](plans/done/2026-08-23-m6-admin-console.md) | 归档 | 查 M6 是否关闭 | M6 完成证据 |
| [plans/done/2026-08-23-m7-contract-gaps.md](plans/done/2026-08-23-m7-contract-gaps.md) | 归档 | 查 M7 是否关闭 | M7 完成证据 |
| [plans/done/2026-08-23-m8-settings-ia.md](plans/done/2026-08-23-m8-settings-ia.md) | 归档 | 查 M8 是否关闭 | M8 完成证据 |
| [plans/done/2026-08-24-m9-web-and-mcp.md](plans/done/2026-08-24-m9-web-and-mcp.md) | 归档 | 查 M9 是否关闭 | M9 完成证据 |
| [plans/done/2026-08-25-postgres-backend.md](plans/done/2026-08-25-postgres-backend.md) | 归档 | 查预发后端是否接到 Postgres | Argon2 与 `promptark` 库 |
| [plans/done/2026-08-25-oauth-clients.md](plans/done/2026-08-25-oauth-clients.md) | 归档 | 查客户端 OAuth 是否接到登录弹窗 | Google / GitHub；Refresh 不进 Web Storage |
| [plans/done/2026-08-25-account-surface.md](plans/done/2026-08-25-account-surface.md) | 归档 | 查账号面剩余行是否接到设置 | 作者主页、我的发布、下载保留作者 |
| [plans/done/2026-08-25-library-sync.md](plans/done/2026-08-25-library-sync.md) | 归档 | 查个人库云同步是否关闭 | 立即同步、较新者胜、浏览器账号库 |
| [plans/done/2026-08-25-auto-update.md](plans/done/2026-08-25-auto-update.md) | 归档 | 查自动更新安装是否关闭 | GitHub Releases；updater 排队安装 |
| [templates/adr.md](templates/adr.md) | 模板 | 写 ADR | ADR 模板 |
| [templates/capability-spec.md](templates/capability-spec.md) | 模板 | 写新能力规格 | 规格模板 |
| [templates/implementation-plan.md](templates/implementation-plan.md) | 模板 | 写模块计划 | 计划模板 |
| [superpowers/specs/2026-08-22-documentation-system-design.md](superpowers/specs/2026-08-22-documentation-system-design.md) | 归档 | 追溯文档方案来源 | 文档体系设计记录 |
