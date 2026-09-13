# 文档索引

Agent 只读本表，再打开需要的文件。没出现在本表的文档视为不存在。

图例：`现行` = 今天为真；`目标` = 已指定尚未实现；`模板` = 复制用；`归档` = 非现行真相。

| 路径 | 状态 | 何时读 | 一句话 |
|---|---|---|---|
| [plans/2026-09-13-mcp-search.md](plans/2026-09-13-mcp-search.md) | 验收 | MCP 搜索性能与慢请求隔离 | 27 项、5 万条基准和真实库只读已验，release 已更新 |
| [plans/2026-09-13-content-metrics.md](plans/2026-09-13-content-metrics.md) | 验收 | 下载/收藏展示与个人作品排序 | 497 项前端、10 项后端及浏览器已验，服务和客户端已更新 |
| [plans/2026-09-13-account-design.md](plans/2026-09-13-account-design.md) | 验收 | 优化账号页展示和资料编辑 | 账号概览与卡片分组、494 项回归及浅深色验收，客户端已更新 |
| [plans/2026-09-13-oauth-completion.md](plans/2026-09-13-oauth-completion.md) | 验收 | OAuth 授权后浏览器停在加载页 | 完成页、16 项 OAuth 与浏览器导航验证通过，8080 已重启 |
| [plans/2026-09-13-google-consent.md](plans/2026-09-13-google-consent.md) | 验收 | Google 登录未显示授权确认 | 显式账号选择/consent；15 项 OAuth 测试及 8080 运行参数已验 |
| [plans/2026-09-13-local-ports.md](plans/2026-09-13-local-ports.md) | 验收 | 修改本机服务端口或兼容旧回调 | 两项目迁移与重启通过；14 项 OAuth、8 项 Flutter 配置测试及桌面构建 |
| [plans/2026-09-13-local-oauth-config.md](plans/2026-09-13-local-oauth-config.md) | 验收 | 接入本机第三方登录凭据 | 本机凭据已验；后续端口迁移兼容旧回调，真实授权待验证 |
| [plans/2026-09-13-global-search.md](plans/2026-09-13-global-search.md) | 验收 | 区分全局搜索与页面筛选 | 本地/广场独立面板；492 项回归、浏览器与 macOS 构建通过 |
| [architecture/decisions/0022-global-search.md](architecture/decisions/0022-global-search.md) | 现行 | 修改搜索位置或交互 | 右上角全局搜索、页面内筛选，临时查找面板边界 |
| [plans/2026-09-13-visual-hierarchy.md](plans/2026-09-13-visual-hierarchy.md) | 验收 | 优化页面设计 | 页头、卡片、侧栏与紧凑编辑器；481 项回归、浅深色浏览器及构建通过 |
| [plans/2026-09-12-thumbnail-cache.md](plans/2026-09-12-thumbnail-cache.md) | 验收 | 大图列表与长文滚动 | 缩略图缓存/摘要、480 项全量加 1 项补验、85 项原生及浏览器构建通过 |
| [plans/2026-09-12-scroll-smoothness.md](plans/2026-09-12-scroll-smoothness.md) | 验收 | 滚动掉帧 | 重复渲染/样式读取消除，474 项前端与构建通过；原生重启复验待完成 |
| [plans/2026-09-12-workbench-continuity.md](plans/2026-09-12-workbench-continuity.md) | 验收 | 优化跨页整理与预览修改 | 471 项前端、跨页浏览器补验与 macOS 构建通过 |
| [plans/2026-09-12-workbench-usability.md](plans/2026-09-12-workbench-usability.md) | 验收 | 优化页面与日常交互 | 四片已提交；467 项前端、83 项原生、浏览器链路与 macOS 构建通过 |
| [plans/2026-09-12-launcher-draft-resume.md](plans/2026-09-12-launcher-draft-resume.md) | 验收 | 启动器切换应用后丢填写 | 草稿恢复、455 项前端和 82 项原生通过；已重启验证入口恢复，物理快捷键待复验 |
| [plans/2026-09-12-argument-variables.md](plans/2026-09-12-argument-variables.md) | 验收 | 导入 argument 参数不能填写 | 默认值与双入口测试通过，桌面逐步填写和最终预览已验 |
| [plans/2026-09-12-local-thumbnails.md](plans/2026-09-12-local-thumbnails.md) | 验收 | 本地列表缩略图 | 网格/行缩略图及按需读取已验，桌面包已构建、重启待确认 |
| [plans/2026-09-12-downloaded-images.md](plans/2026-09-12-downloaded-images.md) | 验收 | 下载缺图片与查看大图 | 参考图离线保存、旧副本补图和大图；前端/原生/浏览器/桌面验收 |
| [plans/2026-09-12-qq-mail.md](plans/2026-09-12-qq-mail.md) | 验收 | 配置 QQ / Foxmail SMTP | 快捷配置已验；真实保存待 owner 重认证，测试发信待确认 |
| [plans/2026-09-12-admin-client-style.md](plans/2026-09-12-admin-client-style.md) | 已完成 | 管理端视觉对齐客户端 | 共用 tokens、分组导航、控件统一及隔离浏览器验收 |
| [plans/2026-09-12-launcher-defaults.md](plans/2026-09-12-launcher-defaults.md) | 验收 | 启动器无法打开 | 已修复缺省偏好，按钮/搜索/Esc 原生通过，真实快捷键待确认 |
| [plans/2026-09-09-text-corpus.md](plans/2026-09-09-text-corpus.md) | 验收 | 补充非图片提示词 | 三个 MIT 来源追加 616 条，分类、许可、幂等与分页已验 |
| [plans/2026-09-09-category-counts.md](plans/2026-09-09-category-counts.md) | 已完成 | 广场分类数量 | 首批聚合统计、父子汇总与轻量展示 |
| [plans/2026-09-09-square-performance.md](plans/2026-09-09-square-performance.md) | 已完成 | 广场卡顿与滚动加载 | SQL 有界分页、取消请求、虚拟网格与性能验收 |
| [plans/2026-09-08-prompt-corpus.md](plans/2026-09-08-prompt-corpus.md) | 验收 | 批量筛选公开提示词 | 已导入 21,775 条/122 条带图；数据包、幂等导入与分页已验，原生重启待解锁 |
| [plans/2026-09-08-clear-prompt-content.md](plans/2026-09-08-clear-prompt-content.md) | 验收 | 清空本机内容与关闭演示回填 | 已备份清空广场/本地、保留账号设置；禁用重启回填 |
| [plans/2026-09-08-launcher-preferences.md](plans/2026-09-08-launcher-preferences.md) | 验收 | 设置启动器大小与行为 | 四项偏好及恢复默认；425 项前端、75 项原生通过，调试包已构建 |
| [plans/2026-09-08-launcher-refinement.md](plans/2026-09-08-launcher-refinement.md) | 验收 | 启动器尺寸与主题完善 | 紧凑尺寸/系统主题、419 项前端与 74 项原生测试；调试包已更新 |
| [plans/2026-09-08-business-acceptance.md](plans/2026-09-08-business-acceptance.md) | 验收 | 完整业务链路与远程 CI | 隔离真实后端/存储业务链路通过，修复开发重置遗漏；远程 CI 待验 |
| [plans/2026-09-08-admin-hardening.md](plans/2026-09-08-admin-hardening.md) | 验收 | 管理端五项补齐 | 列表状态、持久化 AI/视觉审核、存储对账和隔离 E2E 已本机验收 |
| [../deploy/README.md](../deploy/README.md) | 现行 | 集中验收、部署和发行配置 | 一键回归、独立开发服务及调试/正式构建边界 |
| [plans/2026-09-08-local-release-readiness.md](plans/2026-09-08-local-release-readiness.md) | 验收 | P4–P6 本地收尾 | 本机回归/恢复/打包已验；远程 CI、生产签名与跨平台待验 |
| [plans/2026-09-08-publish-sync-feedback.md](plans/2026-09-08-publish-sync-feedback.md) | 现行 | P3c 发布与同步反馈 | 请求隔离、分阶段反馈、队列重试；406 项前端测试已验 |
| [plans/2026-09-08-collection-feedback.md](plans/2026-09-08-collection-feedback.md) | 现行 | P3b 合集加载与成员反馈 | 请求隔离与只读重试；400 项测试及 macOS 调试构建已验 |
| [plans/2026-09-08-client-operation-feedback.md](plans/2026-09-08-client-operation-feedback.md) | 现行 | P3a 保存与复制反馈 | 写入/刷新分离、只读重试和保存快捷键；392 项及原生构建已验 |
| [plans/2026-09-08-media-reclaim.md](plans/2026-09-08-media-reclaim.md) | 现行 | P1e 云端附件回收 | 手动单项回收；历史引用与并发保护、失败重试已验 |
| [plans/2026-09-08-collection-assets.md](plans/2026-09-08-collection-assets.md) | 现行 | P1d 合集成员公开附件 | 已验逐项选择、成员关联、人工审核与原子导入 |
| [plans/2026-09-08-mcp-square.md](plans/2026-09-08-mcp-square.md) | 现行 | P2 MCP 广场与接入 | 已验收：默认本地、显式联网工具、分页与原生配置生成 |
| [architecture/decisions/0021-mcp-square-opt-in.md](architecture/decisions/0021-mcp-square-opt-in.md) | 现行 | 调整 MCP 网络边界 | 独立工具只读匿名广场，不借用桌面会话 |
| [plans/2026-09-08-publication-assets.md](plans/2026-09-08-publication-assets.md) | 现行 | P1c 单条投稿附件 | 显式选择、强制人工审核、按可见性代理与原子下载已验 |
| [plans/2026-09-08-private-media.md](plans/2026-09-08-private-media.md) | 现行 | P1 附件传输基础 | 私有上传、鉴权下载和真实 MinIO 往返已验 |
| [plans/2026-09-08-private-asset-sync.md](plans/2026-09-08-private-asset-sync.md) | 现行 | P1b 私有附件手动同步 | 已验显式选择、私有引用、重试复用与本地补齐 |
| [specs/media/spec.md](specs/media/spec.md) | 现行 | 附件传输权限合同 | 私有鉴权、稿件公开代理及保守手动回收 |
| [plans/2026-09-08-local-completion.md](plans/2026-09-08-local-completion.md) | 验收 | 不依赖外部凭据的后续完善 | 本地切片完成；汇总 P0–P6 实测与范围外能力 |
| [plans/2026-09-08-download-feedback.md](plans/2026-09-08-download-feedback.md) | 现行 | 下载反馈与重复下载 | 来源 ID 持久状态、入口防重复与固定轻提示 |
| [plans/2026-09-08-disable-worklog-gate.md](plans/2026-09-08-disable-worklog-gate.md) | 现行 | 关闭 WorkLog 提交拦截 | 用户明确停用本仓库 IDE 评审钩子，保留测试与 CI |
| [plans/2026-09-08-community-import.md](plans/2026-09-08-community-import.md) | 现行 | 导入公开提示词样本 | 有来源许可的本机追加导入与参考图 |
| [plans/2026-09-08-detail-image-preview.md](plans/2026-09-08-detail-image-preview.md) | 现行 | 查看详情样图排版 | 两张摄影样图、来源与限宽阅读；349 项测试及 macOS 构建 |
| [plans/2026-09-08-browse-refinement.md](plans/2026-09-08-browse-refinement.md) | 现行 | 优化广场与本地浏览 | 准确计数与标题、筛选恢复和合并页头；347 项测试及原生广场验收 |
| [plans/2026-09-08-client-assets.md](plans/2026-09-08-client-assets.md) | 现行 | 图片附件和列表优化 | 本地附件、预览备份与云端隔离；343 项前端测试和 macOS 构建验收 |
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
| [plans/2026-09-08-workspace-pages.md](plans/2026-09-08-workspace-pages.md) | 现行 | 减少客户端弹窗 | 页面化已验收；333 项测试与真实 macOS 返回验证，待提交评审门禁 |
| [architecture/decisions/0020-workspace-pages.md](architecture/decisions/0020-workspace-pages.md) | 现行 | 决定弹窗与页面边界 | 编辑、详情、登录等页面化，仅必要确认使用对话框 |
| [plans/2026-09-08-desktop-refinement.md](plans/2026-09-08-desktop-refinement.md) | 现行 | 查全客户端视觉细化验收 | 统一主壳、弹窗、设置与启动器；328 项测试及浏览器/原生检查 |
| [plans/2026-09-08-settings-page.md](plans/2026-09-08-settings-page.md) | 现行 | 查设置页面验收 | 全窗口设置、主题卡、搜索与返回；328 项测试及桌面包验收 |
| [architecture/decisions/0019-settings-page.md](architecture/decisions/0019-settings-page.md) | 现行 | 修改设置呈现方式 | 取代固定弹窗，采用应用内完整设置页面 |
| [plans/2026-09-08-client-polish.md](plans/2026-09-08-client-polish.md) | 现行 | 优化客户端日常操作 | 账号入口等六项已实现，325 项测试及原生账号入口已验 |
| [plans/2026-09-08-sidebar-resize.md](plans/2026-09-08-sidebar-resize.md) | 现行 | 修复客户端侧栏拖拽 | 有界调宽与折叠保留，浏览器及真实桌面已验 |
| [plans/2026-09-07-admin-risk.md](plans/2026-09-07-admin-risk.md) | 目标 | 实现举报与安全规则 | 举报事务闭环、受限导出和可测试规则 |
| [plans/2026-09-07-admin-moderation.md](plans/2026-09-07-admin-moderation.md) | 目标 | 实现自动审核 | 发布限额、真实初筛、转人工与原子上架 |
| [plans/2026-09-07-admin-ai.md](plans/2026-09-07-admin-ai.md) | 目标 | 实现 AI 模型与审核 Skills | 加密配置、有界外部调用、版本化路由和人工降级 |
| [plans/2026-09-07-admin-mail.md](plans/2026-09-07-admin-mail.md) | 目标 | 实现邮件服务 | 加密 SMTP、显式测试、持久化投递和重试 |
| [plans/2026-09-08-admin-identity.md](plans/2026-09-08-admin-identity.md) | 目标 | 实现注册、邀请与找回 | 一次性邮箱验证、角色保护、注册门禁与三端入口 |
| [plans/2026-09-08-admin-site.md](plans/2026-09-08-admin-site.md) | 目标 | 实现站点配置和公告 | 版本化配置、真实投稿门禁、跨端公告 |
| [plans/2026-09-08-admin-mock-billing.md](plans/2026-09-08-admin-mock-billing.md) | 目标 | 实现模拟账单管理 | 独立持久化、幂等订单、测试兑换码 |
| [plans/2026-09-08-admin-operations.md](plans/2026-09-08-admin-operations.md) | 目标 | 实现概览、审计和系统状态 | 真实口径、失败追踪、限时探测和隔离恢复 |
| [plans/2026-09-08-admin-catalog-migration.md](plans/2026-09-08-admin-catalog-migration.md) | 目标 | 迁移字典引用 | 公开元数据迁移、历史重定向与并发保护 |
| [plans/2026-09-08-admin-oauth-verification.md](plans/2026-09-08-admin-oauth-verification.md) | 目标 | 验证登录配置 | 版本化重认证、一次性真实授权与失败状态 |
| [plans/2026-09-08-admin-notifications.md](plans/2026-09-08-admin-notifications.md) | 目标 | 高风险通知与日志策略 | 加密渠道、持久化去重、失败重试与显式清理 |
| [plans/2026-09-08-admin-acceptance.md](plans/2026-09-08-admin-acceptance.md) | 目标 | 完整管理台最终验收 | 本地全端已验；真实外部验证与 Git 门禁待完成 |
| [how-to/backend-recovery.md](how-to/backend-recovery.md) | 现行 | 备份或恢复后端实例 | 配套密钥/对象存储、隔离数据库恢复演练 |
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
| [architecture/decisions/0005-ui-source-prompt-ark-prototype.md](architecture/decisions/0005-ui-source-prompt-ark-prototype.md) | 归档 | 追溯主窗口设计源 | 已被 ADR 0015 取代 |
| [architecture/decisions/0015-workbench-visual-system.md](architecture/decisions/0015-workbench-visual-system.md) | 归档 | 追溯首次视觉优化 | 已被 ADR 0016 取代 |
| [architecture/decisions/0016-workbench-frame-and-settings.md](architecture/decisions/0016-workbench-frame-and-settings.md) | 归档 | 追溯固定设置弹窗 | 主工作台由 0017、设置由 0019 取代 |
| [architecture/decisions/0017-screenshot-workbench-frame.md](architecture/decisions/0017-screenshot-workbench-frame.md) | 现行 | 对齐用户 Codex 主窗口截图 | 窗口操作与品牌分行、直线分栏及主题层级；搜索部分由 0022 替代 |
| [architecture/decisions/0006-plan-altitude.md](architecture/decisions/0006-plan-altitude.md) | 现行 | 想一次写完所有逐步任务时 | 计划只写一层深 |
| [architecture/decisions/0007-sqlite-access.md](architecture/decisions/0007-sqlite-access.md) | 现行 | 改本地库访问方式时 | rusqlite 而不是 plugin-sql |
| [architecture/decisions/0008-m5-backend-contract.md](architecture/decisions/0008-m5-backend-contract.md) | 现行 | 接广场或后端前 | 改写 API；邮箱密码；覆盖率 |
| [architecture/decisions/0009-m6-admin-console.md](architecture/decisions/0009-m6-admin-console.md) | 现行 | 做管理台或审核 API 前 | 独立 admin.yaml；admin-web 不进桌面 |
| [architecture/decisions/0010-m7-contract-gaps.md](architecture/decisions/0010-m7-contract-gaps.md) | 现行 | 做已登录收藏、轮换或 admin me 前 | 收藏是账号关系；Refresh 轮换；GET /v1/admin/me |
| [architecture/decisions/0011-web-and-mcp.md](architecture/decisions/0011-web-and-mcp.md) | 归档 | 追溯 Web 与 MCP 初始范围 | Web 独立 SPA 保持；MCP 网络边界由 0021 取代 |
| [architecture/decisions/0012-postgres-backend.md](architecture/decisions/0012-postgres-backend.md) | 现行 | 接 Postgres、Redis、MinIO 或改口令存储时 | 预发存本机 `promptark` 库；Argon2id |
| [architecture/decisions/0013-oauth-google-github.md](architecture/decisions/0013-oauth-google-github.md) | 现行 | 接 Google / GitHub 登录前 | 选定 Google 与 GitHub；仍不接 QQ/LinuxDo |
| [architecture/decisions/0014-full-product.md](architecture/decisions/0014-full-product.md) | 现行 | 问是否还按第一期冻结 | 剩余工作按完整产品排队 |
| [architecture/decisions/0018-admin-fixed-roles.md](architecture/decisions/0018-admin-fixed-roles.md) | 现行 | 改管理角色和用户操作 | 固定角色、显式 owner 迁移和最后所有者保护 |
| [specs/launcher/spec.md](specs/launcher/spec.md) | 目标 | 做启动器 | 完整键盘填写、窗口生命周期、复制与原应用粘贴 |
| [specs/workbench/spec.md](specs/workbench/spec.md) | 目标 | 做主窗口壳 | 软件内搜索与启动器分离、保存快捷键同步及桌面框架 |
| [specs/library/spec.md](specs/library/spec.md) | 目标 | 做本地 CRUD | 本地提示词 |
| [specs/square/spec.md](specs/square/spec.md) | 目标 | 改广场浏览、下载或收藏 | 广场只读详情、重试与分类下载 |
| [specs/collections/spec.md](specs/collections/spec.md) | 目标 | 做合集 | 合集编辑/删除与全库成员管理 |
| [specs/categories/spec.md](specs/categories/spec.md) | 目标 | 做分类树 | 自定义大/小分类、新建校验与安全删除 |
| [specs/variables/spec.md](specs/variables/spec.md) | 目标 | 做使用向导或渲染 | 变量解析、字面值与特殊名称隔离 |
| [specs/auth/spec.md](specs/auth/spec.md) | 目标 | 改登录或令牌存放 | 邮箱验证、注册门禁和找回；Refresh 进钥匙串 |
| [specs/settings/spec.md](specs/settings/spec.md) | 目标 | 做设置 | 十页完整设置页面、搜索、主题卡与保存反馈 |
| [specs/publish/spec.md](specs/publish/spec.md) | 目标 | 改发布提交 | 提示词及合集成员快照；入队不等于提交审核 |
| [specs/admin/spec.md](specs/admin/spec.md) | 目标 | 做管理台或审核写路径 | 身份/内容/审核/站点运营、权限与真实消费者 |
| [specs/web/spec.md](specs/web/spec.md) | 目标 | 做浏览器工作台 | 账号隔离、保存重试与复制失败反馈；不进桌面包 |
| [specs/mcp/spec.md](specs/mcp/spec.md) | 现行 | 做 MCP | 本地内存索引、多词筛选分页、取消隔离；显式独立广场工具 |
| [specs/sync/spec.md](specs/sync/spec.md) | 目标 | 做个人库云同步 | 完整分类/合集/成员/软删除往返与时间归一化 |
| [specs/billing/spec.md](specs/billing/spec.md) | 目标 | 做预发账单或兑换 | 独立持久化模拟订单/权益、测试码与真实权益隔离 |
| [specs/documentation/spec.md](specs/documentation/spec.md) | 现行 | 改 docs-check 或索引规则 | 文档门禁合同 |
| [reference/test-gates.md](reference/test-gates.md) | 现行 | 加测试或 CI | 分阶段门禁 |
| [reference/openapi/square.yaml](reference/openapi/square.yaml) | 现行 | 改广场 API 时 | M5 广场 / 登录 / 发布合同 |
| [reference/openapi/admin.yaml](reference/openapi/admin.yaml) | 现行 | 改管理 API 时 | M6 管理合同；/v1/admin |
| [reference/quality.md](reference/quality.md) | 现行 | 评审标准含糊时 | 质量约定 |
| [how-to/local-dev.md](how-to/local-dev.md) | 现行 | 想在本机验证 | npm test / npm run dev / tauri dev / 备份恢复 |
| [how-to/mcp-clients.md](how-to/mcp-clients.md) | 现行 | 让其他智能体搜索提示词 | 独立 stdio、原生配置生成、本地三工具与可选广场三工具 |
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
| [changes/admin-complete/proposal.md](changes/admin-complete/proposal.md) | 目标 | 确认完整管理台范围 | A01–A19；补齐原型 12 页、AI 审核与风控 |
| [plans/2026-09-07-admin-catalog.md](plans/2026-09-07-admin-catalog.md) | 现行 | 复验远端分类与模型管理 | 两级字典、版本审计、引用保护和公共消费端；引用迁移未实现 |
| [plans/README.md](plans/README.md) | 现行 | 准备写或找计划 | 计划目录规则 |
| [plans/program.md](plans/program.md) | 现行 | 问总顺序和依赖 | 程序计划 |
| [plans/status.md](plans/status.md) | 现行 | 问现在做到哪 | 只写今天为真的状态 |
| [plans/2026-09-05-functional-repair.md](plans/2026-09-05-functional-repair.md) | 现行 | 修复分类与跨端功能缺口 | 功能复审队首；旧计划关闭不等于验收完成 |
| [plans/2026-09-07-launcher-audit.md](plans/2026-09-07-launcher-audit.md) | 现行 | 修复或复验启动器 | 核心修复与逐项验收；辅助功能成功路径待授权复验 |
| [plans/2026-09-07-app-icon.md](plans/2026-09-07-app-icon.md) | 现行 | 更换应用图标 | 方舟品牌源图、桌面打包资源与小尺寸验收 |
| [plans/2026-09-07-anonymous-placeholders.md](plans/2026-09-07-anonymous-placeholders.md) | 现行 | 修复空花括号无法填写 | 截图回归、匿名空位逐项填写与代码边界 |
| [plans/2026-09-07-launcher-compact-dismiss.md](plans/2026-09-07-launcher-compact-dismiss.md) | 现行 | 调整启动器密度、位置及复制后抢焦点 | 填写与结果等高、稳定上方锚点与完成后退场 |
| [plans/2026-09-07-shortcut-recorder.md](plans/2026-09-07-shortcut-recorder.md) | 现行 | 修复快捷键无法按键录入 | 三项组合录制、取消导航与全局回调保护 |
| [plans/2026-09-07-search-entrypoints.md](plans/2026-09-07-search-entrypoints.md) | 现行 | 修复搜索入口混淆及快捷键显示 | 软件内搜索聚焦与启动器保存值即时同步 |
| [plans/2026-09-07-scrollbars-focus.md](plans/2026-09-07-scrollbars-focus.md) | 现行 | 统一桌面滚动条及控件描边 | 双窗口滚动样式、安全间距与单层键盘焦点 |
| [plans/2026-09-07-category-actions.md](plans/2026-09-07-category-actions.md) | 现行 | 修复分类新建和删除 | 明确入口、重复校验、内容保留与删除同步 |
| [plans/2026-09-07-custom-root-categories.md](plans/2026-09-07-custom-root-categories.md) | 现行 | 新增本地大分类 | 可选父级、两级关系往返与广场隔离 |
| [plans/2026-09-07-admin-oauth-settings.md](plans/2026-09-07-admin-oauth-settings.md) | 现行 | 管理端配置第三方登录 | Google/GitHub 热配置、密钥保护与管理台视觉整理 |
| [plans/2026-09-07-admin-complete.md](plans/2026-09-07-admin-complete.md) | 目标 | 开始完整管理端开发 | 九阶段依赖与退出标准；覆盖原型及新增运营需求 |
| [plans/2026-09-07-admin-security.md](plans/2026-09-07-admin-security.md) | 现行 | 复验管理安全首个子切片 | 初始化不覆盖、本人改密、全会话撤销与事务审计；不等于安全阶段全部完成 |
| [plans/2026-09-07-admin-users.md](plans/2026-09-07-admin-users.md) | 现行 | 复验角色权限和用户管理 | 显式 owner 迁移、分页检索、详情、停用与撤权；不等于完整身份模块 |
| [plans/2026-09-07-admin-navigation-throttle.md](plans/2026-09-07-admin-navigation-throttle.md) | 现行 | 复验管理基础 1c | 认证节流、地址导航、会话过期与草稿保护 |
| [plans/2026-09-07-admin-reviews.md](plans/2026-09-07-admin-reviews.md) | 现行 | 复验审核运营 3a | 分页检索、驳回理由、事务历史与逐项批量结果 |
| [plans/2026-09-07-admin-content.md](plans/2026-09-07-admin-content.md) | 现行 | 复验广场内容管理 3b | 版本化展示编辑、上下架、回收站和公开访问边界 |
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
| [plans/2026-08-25-win-linux-prefs.md](plans/2026-08-25-win-linux-prefs.md) | 归档 | 查 Windows / Linux 开机启动与托盘怎么做的 | 未验证不得勾 QA |
| [plans/2026-08-25-preview-billing.md](plans/2026-08-25-preview-billing.md) | 归档 | 查预发账单与兑换怎么做的 | 无密钥不得写成 Pro |
| [plans/2026-08-25-stripe-webhook.md](plans/2026-08-25-stripe-webhook.md) | 归档 | 查 Checkout webhook 入账怎么做的 | 无签名不得写成 Pro |
| [plans/2026-08-25-desktop-version.md](plans/2026-08-25-desktop-version.md) | 归档 | 查桌面包版本怎么与 Tauri 构建对齐 | 不得把 0.0.0 写成已安装版本 |
| [plans/2026-08-25-sync-conflict-copy.md](plans/2026-08-25-sync-conflict-copy.md) | 归档 | 查设置冲突行怎么标明较新者胜 | 不得把已接通策略写成尚未提供 |
| [plans/2026-08-26-sync-status-copy.md](plans/2026-08-26-sync-status-copy.md) | 归档 | 查网络页同步状态怎么标明立即同步 | 不得写成没有云同步 |
| [plans/2026-08-26-keychain-copy.md](plans/2026-08-26-keychain-copy.md) | 归档 | 查钥匙串行怎么在浏览器预览标明现状 | 无 Tauri 不得写本机钥匙串 |
| [plans/2026-08-26-keep-local-conflict.md](plans/2026-08-26-keep-local-conflict.md) | 归档 | 查冲突时保留本地怎么接到立即同步 | 不得覆盖该条本机正文 |
| [plans/2026-08-27-web-billing.md](plans/2026-08-27-web-billing.md) | 归档 | 查浏览器账单入口怎么接到预发 API | 不得声称商店上架 |
| [plans/2026-08-27-readme-status.md](plans/2026-08-27-readme-status.md) | 归档 | 查仓库入口怎么标明 M9 与已接通能力 | 不得把已接通写成未接通 |
| [plans/2026-08-27-wifi-image-sync.md](plans/2026-08-27-wifi-image-sync.md) | 归档 | 查仅 Wi-Fi 同步图片怎么接到立即同步 | 非 Wi-Fi 跳过封面仍同步正文 |
| [plans/2026-08-27-auto-sync-queue.md](plans/2026-08-27-auto-sync-queue.md) | 归档 | 查自动同步收藏与发布草稿怎么入队 | 断网入队，不得假装已到达服务器 |
| [plans/2026-08-27-anonymous-download-stats.md](plans/2026-08-27-anonymous-download-stats.md) | 归档 | 查匿名下载统计怎么按开关上报 | 默认关；只发条目 id；不得静默上报 |
| [plans/2026-08-27-manual-proxy.md](plans/2026-08-27-manual-proxy.md) | 归档 | 查手动代理怎么接到本机请求 | 空则跟随系统；浏览器预览不走该代理 |
| [plans/2026-08-29-workbench-shell-wiring.md](plans/2026-08-29-workbench-shell-wiring.md) | 归档 | 查工作台已画出控件怎么接到行为 | 模型筛选、最近收藏、右键、语言、建议 |
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
| [plans/modules/admin.md](plans/modules/admin.md) | 现行 | 查看管理台范围与计划入口 | 区分旧 M6 基线与完整后台待实现目标 |
| [plans/modules/web.md](plans/modules/web.md) | 现行 | 做浏览器工作台 | Web 模块完成态 |
| [plans/modules/mcp.md](plans/modules/mcp.md) | 现行 | 做本机 MCP | MCP 模块完成态，联网边界见 P2 计划 |
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
| [plans/done/2026-08-25-win-linux-prefs.md](plans/done/2026-08-25-win-linux-prefs.md) | 归档 | 查 Windows / Linux 开机启动与托盘是否关闭 | 已写出行为；发行 QA 仍跳过 |
| [plans/done/2026-08-25-preview-billing.md](plans/done/2026-08-25-preview-billing.md) | 归档 | 查预发账单与兑换是否关闭 | status、兑换、测试 Checkout |
| [plans/done/2026-08-25-stripe-webhook.md](plans/done/2026-08-25-stripe-webhook.md) | 归档 | 查 Checkout webhook 入账是否关闭 | 签名校验后才标 Pro |
| [plans/done/2026-08-25-desktop-version.md](plans/done/2026-08-25-desktop-version.md) | 归档 | 查桌面包版本是否与 Tauri 构建对齐 | package.json 与 Cargo 同号 |
| [plans/done/2026-08-25-sync-conflict-copy.md](plans/done/2026-08-25-sync-conflict-copy.md) | 归档 | 查设置冲突行是否标明较新者胜 | 不得把已接通策略写成尚未提供 |
| [plans/done/2026-08-26-sync-status-copy.md](plans/done/2026-08-26-sync-status-copy.md) | 归档 | 查网络页同步状态是否标明立即同步 | 不得写成没有云同步 |
| [plans/done/2026-08-26-keychain-copy.md](plans/done/2026-08-26-keychain-copy.md) | 归档 | 查钥匙串行是否在浏览器预览标明现状 | 无 Tauri 不得写本机钥匙串 |
| [plans/done/2026-08-26-keep-local-conflict.md](plans/done/2026-08-26-keep-local-conflict.md) | 归档 | 查冲突时保留本地是否接到立即同步 | 不得覆盖该条本机正文 |
| [plans/done/2026-08-27-web-billing.md](plans/done/2026-08-27-web-billing.md) | 归档 | 查浏览器账单入口是否接到预发 API | 不得声称商店上架 |
| [plans/done/2026-08-27-readme-status.md](plans/done/2026-08-27-readme-status.md) | 归档 | 查仓库入口是否标明 M9 与已接通能力 | 不得把已接通写成未接通 |
| [plans/done/2026-08-27-wifi-image-sync.md](plans/done/2026-08-27-wifi-image-sync.md) | 归档 | 查仅 Wi-Fi 同步图片是否接到立即同步 | 非 Wi-Fi 跳过封面仍同步正文 |
| [plans/done/2026-08-27-auto-sync-queue.md](plans/done/2026-08-27-auto-sync-queue.md) | 归档 | 查自动同步收藏与发布草稿是否入队 | 断网入队，不得假装已到达服务器 |
| [plans/done/2026-08-27-anonymous-download-stats.md](plans/done/2026-08-27-anonymous-download-stats.md) | 归档 | 查匿名下载统计是否按开关上报 | 默认关；只发条目 id；不得静默上报 |
| [plans/done/2026-08-27-manual-proxy.md](plans/done/2026-08-27-manual-proxy.md) | 归档 | 查手动代理是否接到本机请求 | 空则跟随系统；浏览器预览不走该代理 |
| [plans/done/2026-08-29-workbench-shell-wiring.md](plans/done/2026-08-29-workbench-shell-wiring.md) | 归档 | 查工作台已画出控件是否接到行为 | 模型筛选、最近收藏、右键、语言、建议 |
| [templates/adr.md](templates/adr.md) | 模板 | 写 ADR | ADR 模板 |
| [templates/capability-spec.md](templates/capability-spec.md) | 模板 | 写新能力规格 | 规格模板 |
| [templates/implementation-plan.md](templates/implementation-plan.md) | 模板 | 写模块计划 | 计划模板 |
| [superpowers/specs/2026-08-22-documentation-system-design.md](superpowers/specs/2026-08-22-documentation-system-design.md) | 归档 | 追溯文档方案来源 | 文档体系设计记录 |
