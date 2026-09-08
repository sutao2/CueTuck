# 站点配置与公开消费切片

| 字段 | 值 |
|---|---|
| 状态 | 设计完成，实施中 |
| 范围 | A10；复用既有匿名浏览和注册策略，不复制注册开关 |

## 数据与兼容方案

- 增量 singleton site_configuration 保存 revision、name（1–60 字）、description（最多 300 字）、logo_url（可空，仅 HTTPS）、support_email（可空）、publishing_open（默认 true）、announcement（纯文本最多 2000 字）、announcement_start/end（可空 UTC RFC3339，起早于止）。不允许 HTML/script/logo data URL；远程 logo 由用户浏览器加载，referrerpolicy=no-referrer，不由后端下载。
- square_public 仍是 settings 的唯一值；新配置响应合并读取，写入时同事务更新。旧 PUT settings 仅变更 square_public，但也取得 site 配置锁、递增站点 revision、验证会话和写审计，避免旧客户端绕过新版本保护。
- registration_open 仍由 identity_policy 唯一管理；站点页链接注册与邀请，不再存第二份值。
- owner 新 GET/PUT /v1/admin/site，PUT 包含 revision 与全部字段；旧 revision 拒绝，未知字段拒绝；保存与审计原子一致。公开 GET /v1/site 及广场 catalog 附加 site 返回公开品牌/支持/当前有效公告/投稿门禁，不返回内部邮件配置。
- 投稿事务先取得 site_configuration 的共享锁并检查 publishing_open，再采用原有 actor/policy/rules/catalog 锁顺序；关闭投稿与新提交串行，已提交投稿仍可审核。本地库、下载和启动器不受影响。
- 桌面/Web 复用广场字典请求消费站点信息，展示社区品牌、说明、支持地址及有效公告；默认品牌用于离线/旧服务器，无远端配置不可覆盖本地应用名称。公告到期自动隐藏，不依赖刷新。

## 任务与验证

1. 隔离 PostgreSQL 测试字段/权限/CAS/旧接口/审计回滚/公告时段；直接提交在关闭时拒绝且不入库。
2. 实现增量表与公开/API 消费；保留历史 square_public 和注册状态。
3. 管理表单加载失败禁保存、未保存提醒、保存中锁定、服务端确认版本；支持注册策略导航。
4. 桌面/Web 测试真实展示、到期隐藏、离线不阻塞本地；管理表单失败/冲突回归，宽窄屏浏览器验收。
5. 全端测试与构建、规格/合同/INDEX、正常提交；不因验收给真实用户发公告或关闭当前站点。

## Given/When/Then

- GIVEN 投稿关闭；WHEN 旧客户端或直接 API 提交；THEN 返回 403、不创建投稿，本地编辑仍可用。
- GIVEN 旧站点 revision 或审计失败；WHEN 保存；THEN 不覆盖新设置，所有字段及匿名开关全部回滚。
- GIVEN 公告尚未开始或已经过期；WHEN 请求公开配置或停留页面；THEN 不展示公告；内容仅按文本渲染。

## 验收记录

- [x] API、兼容性与权限事务：两项隔离站点测试及后端全量 119 项通过，包含投稿关闭、旧匿名接口递增版本、审计故障全回滚与公告时段。
- [x] 管理表单与桌面/Web 消费：管理端 66 项、Web 33 项通过，桌面公告到期/纯文本和本地隔离回归通过；三端构建成功。
- [x] 浏览器：最新本地后端读取站点 revision 0；宽屏截图人工检查，390 宽页面无横向溢出。未发布真实公告或关闭现有站点。
- [ ] 正常提交仍由 WorkLog IDE 评审门禁约束，不绕过。
