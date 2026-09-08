# 管理端全量联调与验收

| 字段 | 值 |
|---|---|
| 状态 | A01–A19 范围代码及本地全端验收完成；真实外部凭据验收、生产演练和 Git 评审门禁未完成 |
| 范围 | A01–A19 最终核对；不扩大为生产上线或新身份提供商 |

## 已发现的联动缺口与修复顺序

1. 用户停用/启用尚未录入原因，与范围合同不符。新增必填原因（去空白后 1–1000 字）、服务端校验、同事务安全审计；旧缺原因请求返回 400，不给敏感写入设默认理由。其他用户操作保持既有兼容行为，当前密码和最后 owner 保护不变。
2. 用户详情仍从进程内读取 mock 权益、页面仍说重启清除。改为与账单一致的 Postgres mock 权益查询；真实权益与模拟权益分开标明。增加写入持久化模拟权益后详情一致的回归，不改现有实例权益。
3. 审查管理员所有导航/API/表单：权限矩阵、直接路由、浏览器返回、错误与重试、分页、未保存离开、键盘和窄屏。消除与现行实现不符的文案，不重新设计无问题的页面。
   - 原生包 CSP 仅允许本地/data 图片，会拦截管理端已支持的 HTTPS 站点标识。仅为 img-src 增加 https:，脚本和连接策略不放宽；组件仍拒绝非 HTTPS/带凭据 URL，并使用 no-referrer。增加原生配置回归检查。
4. 在隔离测试 schema 验证各角色拒绝、配置重启和原有数据保留；执行 backend/admin-web/desktop/web 的测试与构建，原生桌面重新构建并启动，保证新增命令不只存在于源文件。
5. 按 A01–A19 记录对应实现、测试与外部待验证项；本地无法替代 Google/GitHub/SMTP/AI 的真实凭据验证，也不声称图片已经经过视觉模型审核（含图投稿按人工策略处理）。正常提交；WorkLog 拒绝时不绕过、不伪造评审。

## Given/When/Then

- GIVEN owner 或有权运营管理员停用/启用账号；WHEN 原因缺失、空白或过长；THEN 拒绝且账号/会话不变；合法原因与动作同事务审计。
- GIVEN 模拟权益已经持久化；WHEN 重开后端连接或查看用户详情；THEN 与模拟账单一致，真实权益不变，不从旧内存集合得出错误结果。
- GIVEN 匿名、普通用户、审核员、运营管理员及 owner；WHEN 请求不属于该角色的管理页面/API；THEN 直接请求同样拒绝，页面不显示无权操作，401 清空会话，403 不冒充过期。
- GIVEN 三端采用新 API；WHEN 构建、启动和浏览操作；THEN 无缺命令/缺接口；失败状态明确，不对用户数据执行测试性修改。

## 证据

## 功能核对（2026-09-08）

以下均为代码实现与本地验证，不将隔离替身当作真实提供商认证；原型映射以范围提案为准。

| 编号 | 已实现与验证 | 主要回归证据 |
|---|---|---|
| A01 | 初始化不覆盖账号、明确 owner 迁移、固定角色、本人改密、会话撤销、最后所有者与登录节流 | admin_security_tests、admin_users、auth_limits；AccountSecurity / Navigation |
| A02 | 用户分页/筛选/公开详情、带原因启停、退出与重置邮件；真实/模拟权益一致且分开 | admin_users、admin_mock_billing_tests；UserManagement |
| A03 | 注册门禁、一次性邮箱验证、找回、邀请与撤权；三端入口 | identity_tests；IdentityForm / IdentitySettings / WebApp；原生注册页读取未配置 SMTP 状态 |
| A04 | 投稿筛选/分页、原因/历史、原子审核、批量逐项结果 | admin_reviews_tests、persistence_tests；ReviewManagement |
| A05 | 公开元数据、推荐排序、下架恢复、回收站与全公开路径拦截 | admin_content_tests；ContentManagement |
| A06 | 大/小分类与模型维护、引用统计和迁移、历史重定向、客户端字典 | admin_catalog_tests；CatalogManagement / WorkbenchShell / square；原生广场实际加载 |
| A07 | 加密 Google/GitHub 配置、版本重认证、独立授权验证及失效状态 | oauth_verification_tests、oauth profile tests；OAuthSettings；真实凭据待提供 |
| A08 | 加密 SMTP、测试/身份邮件、持久化租约/失败/有限重试 | admin_mail_tests、identity_tests、mail_transport；MailSettings；真实收件箱待验 |
| A09 | 独立模拟订单/权益、幂等动作、测试码批次/额度/有效期/停用/使用记录 | admin_mock_billing_tests、billing；MockBilling 与三端账单；无真实扣费 |
| A10 | 站点字段、限时公告、投稿/匿名门禁及跨端消费 | admin_site_tests；SiteSettings / SiteNotice / WorkbenchShell；原生站点信息与 HTTPS 图片 CSP |
| A11 | 带时间范围和历史缺失口径的真实统计 | admin_operations_tests；Operations；当前实例查询无虚构增长 |
| A12 | 成功与失败脱敏审计、受限导出、依赖状态与恢复手册 | admin_operations_tests；Operations；隔离 PostgreSQL dump/restore 显式通过，正式对象存储切换待验 |
| A13 | 20 个实际页面路由、角色菜单、错误/草稿/确认/分页、会话过期与窄屏 | Navigation 8 项；模块组件测试；20 页 390 像素实际遍历无 alert/水平溢出，1440 像素检查 |
| A14 | 用户举报/本人进度、分派、原因/历史、下架结案与受限导出 | admin_risk_tests；RiskManagement / ReportPanel |
| A15 | 发布额度、结构/重复/敏感初筛、阈值/人工策略与并发保护 | admin_moderation_tests；ModerationSettings；关闭/异常不自动放行，含图转人工 |
| A16 | 加密文本模型配置、已保存样本测试、有界调用及主备/共识路由 | admin_ai_tests、ai_transport；AiSettings；真实模型凭据待提供 |
| A17 | Skills 版本/历史、分类/类型适用、路由和实际判定版本绑定 | admin_ai_tests；AiSettings；人工先到与配置变化不被旧结果覆盖 |
| A18 | 可编辑安全规则、敏感词、命中解释与共享初筛/举报复核 | admin_risk_tests、admin_moderation_tests；RiskManagement |
| A19 | 风险邮件/签名 Webhook、目标加密、去重限额、投递历史/有限重试、显式运行日志留存 | admin_notifications_tests 6 项；NotificationSettings 5 项；真实目标未配置，默认关闭且不清理 |

## 全端验收

- 后端 `cargo test`：138 项常规测试通过；恢复用例默认忽略，前序已在隔离数据库显式通过。所有者账号、密码和业务数据未被测试重置。
- 管理端：81 项测试及构建通过。真实浏览器 20 页无加载错误；写路径主要用隔离自动化验证，不在真实实例批量改用户、发邮件或授权外部应用。
- 桌面前端：306 项测试及构建通过；原生 Rust 65 项通过、1 项性能基准默认忽略。原生构建保留 9 条既有未使用警告，未做无关清理。
- Web：33 项测试与构建通过；桌面管理 API/公开 API 合同映射同步通过。
- 已重新生成 macOS 调试应用包并实际启动，不是只更新开发服务器。新版广场显示后台站点说明、远端条目；原生注册入口显示 SMTP 尚未启用且不允许发送。本地两条提示词保留。未签生产发行包、未运行 Windows/Linux 原生验收。
- 管理入口 `http://127.0.0.1:5174/`，后端 `127.0.0.1:8787`。外部服务仍未配置，不会发送真实通知或启用自动清理。

## 尚需外部条件

1. owner 配置实际 Google/GitHub 应用及回调、SMTP 和指定收件箱、文本模型凭据、通知目标后，执行各页显式测试并记录真实结果。含图投稿目前明确转人工，没有实现视觉模型图片检测，不能以补凭据声称已做图片审核。
2. 生产部署、正式数据库+对象存储+密钥配套切换、跨平台发行另需对应环境与授权；本轮未操作生产。
3. 所有正常提交尝试仍被 WorkLog 的 IDE 暂存评审门禁拒绝。代码与规格已暂存，未绕过或伪造评审，尚无本轮新提交/推送。需在 IDE 对最终暂存内容完成评审。
