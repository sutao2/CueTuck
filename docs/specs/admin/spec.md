# 管理台

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M6 目标 |
| 第一期 | M0–M4 无管理端；M5 不实现审核写路径 |
| 关联 | [发布](../publish/spec.md) · [认证](../auth/spec.md) · [M6](../../plans/milestones/m6.md) |

## Purpose

给运营人员在独立浏览器里审核广场投稿、查看用户、改运行时设置。不进桌面安装包，不改作者本机 SQLite。

## Requirements

### Requirement: 显式附件回收

系统状态页 MUST 提供配置管理员手动检查、单项非模态确认和失败重试入口，不自动请求清理。取消不发写请求，忙碌时禁止重复执行和站内离开；未知响应不冒充成功。对象范围、历史引用保护、并发与备份警示统一遵循 [P1e 计划](../../plans/2026-09-08-media-reclaim.md)，不得扩大到作者本地文件。

### Requirement: 投稿附件人工检查

审核页 MUST 展示该稿显式选中的文件、提供经校验的图片/纯文本预览及下载；附件稿件通过前须确认公开范围。仅审核角色可读取稿件范围文件，不允许访问未选中的作者私有对象。完整权限与故障场景见 [P1c 计划](../../plans/2026-09-08-publication-assets.md)。人工审核不是防病毒保证。

### Requirement: 生效的站点配置

owner MUST 能版本化保存社区名称、说明、HTTPS 标识、支持邮箱、匿名浏览/投稿开关及限时纯文本公告。注册策略只引用身份模块的唯一开关。加载失败不允许保存默认值覆盖现有站点；全部站点字段、匿名开关与审计同事务。

#### Scenario: 配置具有真实消费者

- GIVEN 新投稿已关闭或公告尚未开始/已经过期
- WHEN 客户端读取广场配置、停留到公告到期或直接向发布 API 提交
- THEN 公告按时段展示/隐藏；投稿在后端拒绝，而本地编辑、下载及启动器仍能使用
- AND 社区标识/说明/支持邮箱仅用于社区，不覆盖用户本地应用品牌；旧匿名开关接口也使站点 revision 递增，防止并发覆盖
- AND 原生包仅在图片 CSP 中允许 HTTPS 标识，不放宽脚本和 API 连接策略；组件禁非 HTTPS/带凭据地址并使用 no-referrer

### Requirement: 注册策略与管理员邀请

owner MUST 能控制新账号注册、以当前密码确认新邮箱的 admin/reviewer 邀请并撤销待接受邀请。匿名管理员注册和邀请覆盖已有账号 MUST 被拒绝。管理员发起重置只能给目标邮箱排队发验证邮件，不得读取验证码或直接设置他人口令。

#### Scenario: 邀请与重置的权限边界

- GIVEN 邀请已到期、已撤销、重复使用，或邀请者被禁用/撤销 owner
- WHEN 接受邀请，包括随后重新启用邀请者
- THEN 不创建新管理员；账号停用/撤权时永久撤销相应待接受邀请
- AND admin 只能为普通用户发起重置，owner 才可为管理账号发起；重置完成与撤销全部会话同事务

管理台登录页只提供找回密码和接受邀请，不提供公开管理员注册；公开注册及 OAuth 门禁遵循[认证规格](../auth/spec.md)。

### Requirement: 可验证的邮件服务

owner MUST 重新认证后维护加密 SMTP 配置；仅允许公网目标、证书校验的 TLS/强制 STARTTLS，不允许明文降级。保存不发信，显式测试发送固定模板到指定邮箱。投递队列持久化并加密敏感正文，管理列表不返回邮件验证码/正文；失败、过期与重试必须真实可见。

#### Scenario: 邮件不能虚报成功

- GIVEN 配置保存后未测试、SMTP 拒绝或网络超时
- WHEN 查看投递与配置状态
- THEN 只能标记未测试或失败，成功也只代表 SMTP 服务器接受，不代表收件箱送达
- AND 凭据/配置变化不沿用旧版测试状态，重试受权限、期限、次数和并发认领约束

### Requirement: 真实模型审核与版本化 Skills

审核任务 MUST 与投稿快照同事务落库，唯一关联投稿；不在投稿 HTTP 请求中等待外部模型。worker 使用 90 秒租约、认领令牌、过期重领、最多三次和 30 秒倍数退避；过期 worker 不得落判定。人工处置优先，配置变化不沿用旧结论自动上架。配置管理员可分页读取任务状态、有限显式重试；任务成功写入与判定同事务，投稿保留不等于审核完成。后台入口和故障验证见[五项补齐计划](../../plans/2026-09-08-admin-hardening.md)。

模型的 vision 字段默认 false，必须明确授权图片发送；图片与文本共同进入所选 Skill 的视觉模型路由。未授权、类型或限额不符合、完整性错误、模型失败均不能假报通过。带附件的最终公开仍须人工确认，具体传输限制见[附件规格](../media/spec.md)。管理页文本样本测试不冒充视觉模型实测。

owner MUST 能重新认证后维护审核 API 密钥、端点、模型标识和启停，列表及审计不得返回密钥/密文。保存配置不等于连通性验证，修改配置清除旧测试状态。发送前 MUST 校验 HTTPS/DNS 公网目标、禁代理/重定向、限制超时/重试/响应体。Skills 保存追加版本，判定关联实际版本；测试只用已保存配置且明确外部发送。

#### Scenario: 失败不冒充结果

- GIVEN 主备/共识模型缺失、超时或返回非法结构
- WHEN 运行投稿审核或显式测试
- THEN 返回实际失败和人工复核状态，不生成伪造分数或成功提示
- AND 原始响应/密钥不回显，本地私有内容不发送

#### Scenario: 历史与并发保护

- GIVEN 模型调用期间人工已处置投稿或配置版本已变化
- WHEN 旧 AI 结果返回
- THEN 保留真实历史，不覆盖人工结论或按旧配置自动上架；结果落库、上架与审计原子完成

### Requirement: 可配置自动审核

owner MUST 能配置自动审核开关、每作者 24 小时投稿限额、低风险自动通过/高风险转人工阈值，以及重复、结构、敏感词、图片和 AI 检查。默认关闭。写入带版本、事务审计，页面不可将加载失败的默认值保存为真实配置。

#### Scenario: 真实初筛和不可用依赖

- GIVEN 策略要求 AI/图片检查，但对应依赖不可用
- WHEN 用户发布公开快照
- THEN 记录实际初筛配置/规则版本、原因并转人工，不能假称模型或图片审核通过
- AND 仅显式不要求外部检测且所有本地检查完成的低风险投稿可标为本地规则自动通过，不冒充 AI

#### Scenario: 原子审核与限额

- GIVEN 同作者并发投稿或自动通过时审计失败
- WHEN 请求提交
- THEN 配额串行检查，不可超额；投稿、初筛结果、自动上架和系统审核历史同事务，失败全部回滚

### Requirement: 举报与确定性安全规则

owner/admin MUST 能检索举报、分派给有效运营管理员、记录处理理由、驳回或下架结案。写入携带 revision，已结案不能再次处置；下架与案件/历史/审计同事务。用户只能举报在线广场内容，读取自己的状态和结案原因；同人同目标未结案去重、每日最多 20 件。后台不能浏览私有库。

#### Scenario: 处置冲突与失败

- GIVEN 旧版本、已关闭案件或审计写入失败
- WHEN 提交下架结案
- THEN 拒绝冲突或回滚全部写入，不重复下架；公开访问遵循已有下架规则

#### Scenario: 规则配置与测试

- GIVEN 运营人员配置规则分类、字面关键词、分数和启停
- WHEN 保存当前版本或测试公开文本
- THEN 版本冲突拒绝；测试返回启用规则的命中解释，不使用用户正则或外部服务
- AND 无命中只表示未触发当前规则，不表示 AI 已审核或内容安全；导出限制 500 条必要元数据并审计，不含正文或秘密

### Requirement: 广场分类与模型字典

owner/admin MUST 能新增、编辑、排序、启停和安全删除远端分类/模型；分类最多两级且同级不能重名，模型值/分类 ID 创建后不可变。模型字典与审核服务 API 配置是不同能力。后台页面 MUST 有加载/失败反馈、保存防重复、草稿离开保护、删除确认和引用说明。

#### Scenario: 版本、层级与引用保护

- GIVEN 字典已有子分类或被广场条目、投稿历史（含合集成员）引用
- WHEN 删除该项、创建第三级/循环关系、重复名称或提交旧 revision
- THEN 拒绝操作且不改变现有数据；并发写入只接受当前版本
- AND 权限和会话在事务内复核；新增引用与字典修改串行化；业务和审计同事务，审计失败回滚

#### Scenario: 修改和重启生效

- GIVEN 运营人员修改名称、排序或停用分类/模型
- WHEN 管理台、广场或发布表单重新加载，或后端重启
- THEN 使用保存后的远端字典，停用父级不返回其子级，删除墓碑不复活；已发布内容不自动下架
- AND 停用分类不可用于新投稿；已知停用模型不可用于新投稿；已有自由文本模型兼容保留
- AND 本地分类不被远端覆盖；管理内容表单可保留原有停用值，但不能改为其他停用分类

### Requirement: 独立于桌面

管理端 MUST 是独立 Web 应用。桌面安装包 MUST NOT 包含管理源码或管理路由。启动器 MUST NOT 请求管理接口。

#### Scenario: 桌面包不含管理

- GIVEN 构建桌面客户端
- WHEN 检查安装包与 `desktop/` 依赖
- THEN 产物不含 `admin-web` 源码
- AND 不含管理审核路由

#### Scenario: 启动器不请求管理

- GIVEN 启动器窗口已打开
- WHEN 用户搜索或选择本地条目
- THEN 不发起管理接口请求

### Requirement: 管理员身份

管理写路径 MUST 要求带管理员角色的 Access。普通用户 Access MUST 被拒绝且不改发布状态。管理端 Refresh MUST NOT 进入 Web Storage。

#### Scenario: 普通令牌不能审核

- GIVEN 一条 `pending` 发布
- AND 调用方持有无管理员角色的 Access
- WHEN 请求通过或驳回
- THEN 请求失败
- AND 该条仍为 `pending`

#### Scenario: 管理端不持久化 Refresh

- GIVEN 运营人员在浏览器管理端登录成功
- WHEN 检查 Web Storage
- THEN 不出现 Refresh
- AND 关闭标签后须重新登录

#### Scenario: 管理端列出已配置提供商

- GIVEN `GET /v1/session/oauth/providers` 返回 `google`
- WHEN 管理员打开登录页
- THEN 可见 Google 登录
- AND 不出现 QQ / LinuxDo

#### Scenario: 管理端 OAuth 不写 Refresh

- GIVEN 管理端用 Google 登录成功
- WHEN 检查 Web Storage
- THEN 不出现 Refresh

#### Scenario: 查询管理员身份

- GIVEN 调用方持有管理员 Access
- WHEN 请求 `GET /v1/admin/me`
- THEN 返回该账号邮箱与管理员角色

- GIVEN 调用方持有普通用户 Access
- WHEN 请求 `GET /v1/admin/me`
- THEN 请求失败

### Requirement: 审核发布

管理员 MUST 能列出待审发布，并能通过或驳回。审核结果 MUST NOT 锁定或改写作者本地正文。

#### Scenario: 列出待审

- GIVEN 至少一条 `pending` 发布
- WHEN 管理员打开审核列表
- THEN 能看到该条的标识与来源摘要

#### Scenario: 通过不改本地

- GIVEN 一条 `pending` 发布，作者本地仍可编辑
- WHEN 管理员将其标为通过
- THEN 远端状态不再是 `pending`
- AND 作者本地该条仍可编辑且正文未被远端覆盖

#### Scenario: 审核幂等与事务

- GIVEN 一条待审发布和两个并发审核请求
- WHEN 请求给出相同结果或相反结果
- THEN 相同结果可重试且只上架一份，相反结果只接受第一个并对后者返回 409
- AND 审核状态与上架在同一事务提交，上架失败时仍为待审
- AND 管理台提交中禁用重复操作，失败保留条目并展示错误

#### Scenario: 驳回不删本地

- GIVEN 一条 `pending` 发布
- WHEN 管理员将其驳回
- THEN 远端状态为驳回
- AND 作者本地该条仍在库中

### Requirement: 可追溯审核运营

审核列表 MUST 支持关键词、作者、状态、提交日过滤及分页。新投稿记录真实提交时间，旧投稿没有记录时 MUST NOT 用迁移时间冒充。驳回 MUST 让管理员填写原因；兼容旧客户端无请求体时明确记录「旧客户端未提供原因」。审核历史、审计、状态与上架 MUST 同事务提交。

#### Scenario: 驳回与幂等历史

- GIVEN 待审投稿和两个同结果审核请求
- WHEN 审核人员驳回并填写原因
- THEN 只记录首次结果、操作者、时间与原因；重复请求不覆盖原因；相反结果返回 409
- AND 作者能在自己的发布中看到原因与时间，但不获取审核人员邮箱

#### Scenario: 批量部分失败

- GIVEN 当前页选中的待审投稿，其中一条已被其他人处理
- WHEN 确认批量审核（最多 50 个不同 ID）
- THEN 每项独立事务并返回逐项结果，成功不受其他项影响，失败明确标识，不能显示全部成功
- AND 普通用户无权审核，事务内再次校验审核人员角色及会话，审计失败则回滚

### Requirement: 广场内容运营

仅 owner/admin MUST 能检索、编辑广场展示摘要/分类/模型/推荐/权重和变更 online/offline/trashed 状态。作者标题、正文和合集成员 MUST 只读。写入携带 revision，事务内复核当前角色、会话及版本，并与审计原子提交。状态改变 MUST 填原因；不提供永久删除。

#### Scenario: 下架与回收站

- GIVEN 在线条目且管理人员具有当前版本
- WHEN 下架或移入回收站
- THEN 公开列表、详情、正文、收藏展示和下载计数不再接受该条目；本地副本不变
- AND 恢复回收站条目先回到下架，明确上架后使用相同 ID、原收藏关系及计数

#### Scenario: 并发编辑与审计失败

- GIVEN 两个编辑者打开同一版本，或审计写入故障
- WHEN 提交修改
- THEN 旧版本返回 409 并保留前端草稿；审计失败回滚本次数据修改
- AND 审核员无权写入，重启不覆盖被下架或回收的条目

### Requirement: 用户列表与受控操作

管理端 MUST 支持邮箱/显示名检索、角色/状态过滤、分页和用户详情；详情仅含公开资料、登录来源、投稿计数及权益状态，不含私人库、密码或令牌。不得直接代其他账号改密、删除账号或绑定第三方登录。停用/启用、撤销会话与固定角色变更遵循 [ADR 0018](../../architecture/decisions/0018-admin-fixed-roles.md)，写操作重新校验当前密码、权限与会话，失败不产生成功审计。

#### Scenario: 账号原因与权益口径

- GIVEN 管理人员停用或启用账号
- WHEN 原因缺失、全空白或超过 1000 字
- THEN 拒绝且不修改账号/会话；合法原因去首尾空白后与操作原子审计
- AND 用户详情的模拟权益来自与模拟账单相同的持久化表，重开连接不丢失；真实 Pro 独立展示，不从旧内存状态推断

#### Scenario: 固定角色与安全操作

- GIVEN owner、运营管理员、审核员和普通用户
- WHEN 请求用户管理或配置接口
- THEN 审核员/普通用户无法管理用户，运营管理员不能操作管理人员或修改角色；迁移 owner 后配置仅 owner 可访问
- AND 禁止高危操作本人，最后有效 owner 不能被移除；停用或变更角色与令牌撤销及审计原子完成

#### Scenario: 用户列表检索与失败

- GIVEN 超过一页的用户，或请求失败
- WHEN 检索、翻页、读取详情或确认管理操作
- THEN 服务端过滤/分页且显示总数，失败可重试、不显示假成功，提交中不可重复确认

#### Scenario: 看到邮箱与角色

- GIVEN 库中有管理员与普通用户
- WHEN 管理员打开用户页
- THEN 列表含各自邮箱与角色
- AND 没有改密或删除控件

### Requirement: 运行时设置

管理员 MUST 能改一项已文档化的运行时开关，且对后续广场请求生效。未登录浏览广场 MUST NOT 依赖打开管理端页面。

#### Scenario: 关闭公开广场

- GIVEN 运行时开关允许匿名浏览广场
- WHEN 管理员关闭该开关并保存
- THEN 后续匿名广场列表请求失败或返回空
- AND 本地库与启动器仍全部可用

### Requirement: 本人账号安全

持久化管理端 MUST 提供 `GET /v1/admin/security`（`has_password`、`active_access_count`）、`PUT /v1/admin/security/password`（当前/新密码）和 `DELETE /v1/admin/security/sessions`（本人全设备退出）。成功写操作返回 `{ "signed_out": true }`，撤销本人的所有 Access/Refresh，前端清空登录状态。仅管理员可调用；无数据库返回 503，不伪造结果。

#### Scenario: 改密与全设备退出

- GIVEN 管理员已经登录多个设备
- WHEN 验证正确当前密码后提交 12–128 字符（最多 512 UTF-8 字节）的不同新密码，或明确确认全设备退出
- THEN 口令变更（如有）、本人所有令牌撤销与脱敏成功审计同一事务提交，旧令牌均失效；其他账号不受影响
- AND 请求取得账号锁后重新检查令牌与角色；前端成功回到登录页，失败保留输入可重试

#### Scenario: 不允许错误凭据和越权修改

- GIVEN 当前密码错误、匿名、普通用户、额外指定他人账号字段或 OAuth 账号没有本地密码
- WHEN 请求改密
- THEN 请求被拒绝且不改变口令/会话，不产生成功审计；不回显提交的密码

#### Scenario: 安全页交互

- GIVEN 配置读取失败或保存请求未完成
- WHEN 用户查看或提交账号安全表单
- THEN 读取失败可重试且不以默认状态开放写入；提交中阻止重复请求，退出所有设备有明确确认步骤

回归：首个[安全子切片计划](../../plans/2026-09-07-admin-security.md)中的接口、持久化和组件测试。

## 测试映射

### Requirement: 地址导航与会话失效

列表的筛选及页码 MUST 在同一登录会话中跨页面返回保留，返回时重新读取结果；审核/用户/内容保存已应用筛选，实时筛选控件保存当前条件。只保留显式列表字段，不缓存结果、密码、编辑草稿或令牌到浏览器存储；退出、过期和重新登录清空，旧请求不得重新写入下一会话。实现与场景见[五项补齐计划](../../plans/2026-09-08-admin-hardening.md)。

管理页 MUST 有白名单地址、权限检查和前进/后退支持。刷新可重新登录后恢复目标页；令牌仍只在内存。未保存草稿离开须确认，提交中禁止站内离开。配置读取失败不得以默认值开放保存。

#### Scenario: 直接访问和返回

- GIVEN 登录或未登录用户直接访问用户/配置/安全页地址
- WHEN 完成身份校验或使用浏览器前进/后退
- THEN 显示有权限的目标页；越权地址回审核页并解释，无效地址不请求任意管理路径

#### Scenario: 过期与旧响应

- GIVEN 当前会话接口返回 401，或旧会话请求晚于新登录才返回
- WHEN 处理响应
- THEN 当前会话过期回登录并清除敏感草稿；旧响应不得退出新会话或覆盖新会话数据；403 不误报为退出

#### Scenario: 草稿与失败保护

- GIVEN 表单未保存、写入未完成或设置读取失败
- WHEN 离开、退出或尝试保存
- THEN 未保存须确认，提交中禁止站内离开，读取失败仅提供重试而不保存默认值

### Requirement: 真实概览、审计与系统状态

概览 MUST 区分当前状态、累计计数和近 7/30/90 天新增；未知创建时间不可算作新增，不生成无法验证的增长、收入或本地使用数据。owner/admin 可读概览，owner 可查询审计与系统状态。

#### Scenario: 未知历史与依赖失败

- GIVEN 历史账号缺少创建日期或对象存储返回非 2xx
- WHEN 查看概览或系统状态
- THEN 标注未知历史数；对象存储显示不可用，不以请求发出或配置存在代替连通性
- AND 依赖并行限时探测，未配置单独显示，不返回凭据和服务器 URL

#### Scenario: 失败追踪与导出

- GIVEN 管理/身份/投稿写请求失败，或 owner 查询操作历史
- WHEN 保存失败记录、分页查询或确认导出
- THEN 只记录模板路由、状态、请求 ID 和可确认的操作者；不记录请求正文、路径实参或秘密
- AND 导出最多 500 条脱敏摘要并记录导出行为，页面明确数量和上限，没有删除成功安全审计的操作

#### Scenario: 安全恢复

- GIVEN 配套数据库备份、服务器加密密钥和对象存储备份
- WHEN 准备恢复
- THEN 先在隔离环境验证账号校验、快照、数据数量与密文解密；错误密钥必须失败
- AND 正式恢复须明确授权，不提供任意命令执行或一键重置入口，步骤见恢复手册

### Requirement: 高风险通知与运行日志保留

owner MUST 使用当前密码及 revision 配置通知渠道、阈值、每日额度和日志保留；默认均关闭。Webhook 地址与签名密钥 MUST 加密，管理响应只显示地址主机及是否配置，不返回路径或密钥。公开投稿的已提交风险审计按启用起点采集，不读取私有库；队列与测试入口见管理 API 合同。

#### Scenario: 固定元数据与真实投递状态

- GIVEN 显式启用渠道且有新高风险事件，或 owner 确认测试
- WHEN 通知入队和投递
- THEN 事件+渠道去重，仅发送 ID、来源、分数及时间；邮箱复用加密 SMTP 队列，Webhook 使用安全公网 HTTPS 和 HMAC-SHA256 签名
- AND 每日额度包含测试，超额及依赖不可用有跳过记录；入队不能标送达，accepted 只表示对端接收
- AND 并发 worker 使用租约；最多三次、24 小时期限，失败退避，改配置使旧未完成任务失效；至少一次语义需接收端按稳定 ID 去重

#### Scenario: 显式留存策略不损坏业务数据

- GIVEN 默认未开启保留策略
- WHEN 查询页面或运行后台 worker
- THEN 不清理记录；owner 确认备份并开启后，仅批量清理超过保留期的终态通知、邮件、OAuth 验证和失败请求运行记录
- AND 不清理活动任务、安全操作审计、账号、身份挑战、配置历史、投稿、账单或私有库；无任意删除日志接口

### Requirement: 管理台登录配置

管理端 MUST 使用侧栏区分内容审核、用户、第三方登录与站点设置，登录后确认管理员角色再显示管理界面。Google / GitHub MUST 可配置启用状态、Client ID、Client Secret 与回调地址。配置持久化并立即用于后续 OAuth 请求；数据库配置优先于环境变量，未保存时兼容既有环境配置。

#### Scenario: 保存登录配置

- GIVEN 已登录 owner 填写当前密码、当前配置版本、完整凭据与 HTTPS 回调（本机 loopback 可用 HTTP）
- WHEN 保存并启用提供商
- THEN 后续提供商列表与授权跳转使用新配置，无需重启；重启后配置仍在
- AND 配置、验证状态失效和脱敏审计原子提交；版本冲突保留草稿，不覆盖新配置

#### Scenario: 显式真实授权验证

- GIVEN 已保存且启用的 Google/GitHub 配置
- WHEN owner 点击验证授权并在独立窗口授权
- THEN 通过随机 256 位一次性 state 和固定提供商的真实 token/已验证邮箱查询检查配置，10 分钟过期、不可重放
- AND 此流程不创建用户、绑定账号或签发登录会话，当前管理员不退出；模拟身份不能标记真实成功
- AND 管理页可刷新查看等待/成功/失败/过期/配置变化，凭据或回调修改使旧结果失效；回调完成时再次核对配置及有效 owner
- AND 失败只显示固定原因，无提供商正文、令牌、验证码或测试账号邮箱；整个验证出站最多 25 秒，每响应最多 256 KiB，非 2xx/重定向/未验证邮箱不算成功

#### Scenario: 密钥保护

- GIVEN 某提供商已保存 Client Secret
- WHEN 查询、修改其他字段或关闭后重新打开页面
- THEN 响应只返回 `secret_configured`，不含密钥或密文；空白密钥输入保留原值，保存成功清空输入
- AND 数据库保存 AEAD 密文，独立服务器密钥文件不进入前端、同步或版本库

#### Scenario: 配置校验与权限

- GIVEN 未登录、普通用户或无效参数（未知提供商、不安全回调、启用时缺字段）
- WHEN 读取或保存管理配置
- THEN 拒绝请求且不更改配置；前端失败保留输入、可重试，提交中禁止重复保存

#### Scenario: 停用

- GIVEN 管理员停用已配置的提供商
- WHEN 后续请求提供商列表或发起该授权
- THEN 列表不含该项、授权请求被拒绝；邮箱登录仍可用

回归：`backend` OAuth 配置接口、加密及 Postgres 重启测试；`admin-web` 管理身份与配置页测试。

| 场景 | 测试 |
|---|---|
| 管理合同 | `adminContract.test.js` lists `/v1/admin` paths with admin auth |
| 桌面包不含管理 | `desktop` `packageIsolation.test.js` does not depend on or bundle admin-web |
| 启动器不请求管理 | `LauncherApp.spec.js` does not request admin APIs while searching locally |
| 普通令牌不能审核 | `backend` `regular_token_cannot_review_publication` |
| 管理端不持久化 Refresh | `admin-web` `session.test.js` does not persist refresh in web storage |
| 管理端列出已配置提供商 | `admin-web` `AdminApp.spec.js` shows google on login when providers include google |
| 管理端 OAuth 不写 Refresh | `admin-web` `session.test.js` does not persist refresh in web storage after oauth |
| 查询管理员身份 | `backend` `admin_me_returns_email_and_role`、`admin_me_rejects_regular_access` |
| 列出待审 | `backend` `admin_lists_pending_and_can_approve`；`admin-web` `AdminApp.spec.js` lists pending after login |
| 通过不改本地 | `backend` `admin_lists_pending_and_can_approve` 将远端标为 approved；本切片不写桌面库 |
| 驳回不删本地 | `backend` `admin_rejects_publication` 将远端标为 rejected；本切片不写桌面库 |
| 看到邮箱与角色 | `backend` `admin_lists_user_emails_and_roles`；`admin-web` `AdminApp.spec.js` lists emails and roles without password or delete controls |
| 关闭公开广场 | `backend` `admin_can_close_public_square`；`admin-web` `AdminApp.spec.js` saves the anonymous square setting |
| 重启后设置仍在 | `backend` `publication_favorite_and_settings_survive_postgres` |
