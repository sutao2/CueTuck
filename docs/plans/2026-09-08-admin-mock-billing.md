# 模拟订单、权益与测试兑换码

| 字段 | 值 |
|---|---|
| 状态 | 设计完成，实施中 |
| 范围 | A09；不实现真实扣费、退款或结算 |

## 合同调整

原 mock 仅存进程内存、重启清空；为后台历史与多进程一致性，Postgres 实例改为独立 mock 表持久化。内存测试态保持原行为。真实 accounts.pro/redeem_codes/webhook 不写入模拟数据；PROMPTARK_BILLING_MOCK=1 仍是所有模拟写入的硬门禁。停用 mock 不把模拟权益转换成真实 Pro；既有历史仅供管理只读。

## 数据与权限

- mock_entitlements(email,pro,revision)、mock_orders(id,request_id,actor,email,outcome,reason,created_at)，订单唯一 actor/request_id。outcome=success/failure/cancel/reset/redeem；不伪造收费金额或付款凭证。操作先获取同 schema 的 mock advisory lock，再复核 actor/目标账号、更新模拟权益并写订单与审计，同事务。
- owner/admin 可分页搜索真实与模拟权益、模拟订单及测试码批次；审核员和用户不得访问后台。后台调整需原因与随机幂等 request_id；对相同请求重复返回同一订单，不产生重复账目，参数变化冲突。
- mock_code_batches(id,name,enabled,revision,expires_at,uses_per_code)、mock_codes(id,batch_id,code_hash,hint)、mock_code_uses(code_id,email,order_id,created_at)，同账号同码只能一次，额度按码并发限制。每批生成 1–100 个 OS 随机 128 bit TEST- 前缀码，每码使用额 1–1000，最长 365 天。
- 明码只在生成成功时一次返回；数据库存 SHA-256 摘要和末尾提示，管理列表/审计不返回明码。生成 request_id 去重；若网络丢失码，提示无法恢复，停用批次后重新生成，不提供解密导出。
- 批次支持查询、版本化启停、查看码的使用记录；不删除真实或模拟账单，不复活过期码。
- 客户端增加独立 POST /v1/billing/mock/redeem，仅处理 TEST- 码且只写 mock；原 /redeem 与 webhook 在 mock 下仍拒绝。模拟支付沿用 checkout outcome 并新增可选 request_id 兼容旧客户端；新客户端每次意图一个 ID，网络重试复用，不混入真实请求。
- 对未登录、停用账号、模拟关闭、过期/停用/额度耗尽/重复兑换、错误格式和审计失败明确拒绝，不改变真实 Pro。公开兑换沿用身份频控，防猜码和暴力请求。

## 实施与验收

1. 隔离 PostgreSQL 先覆盖真实权益隔离、重启、幂等冲突、审计失败原子回滚、跨账号、并发最后额度和停用/过期。
2. 增量表、模拟接口与现有 checkout/status 的持久化分支；从未修改既有真实兑换码或支付配置。
3. 后台模拟账单页（权益/订单/批次与用量详情）、原因/确认/分页/错误/草稿/一次性码复制；显著 Mock 标签。
4. 桌面/Web 测试兑换入口和原生命令，明确状态持久化规则；失败不报成功，重试不重复记录。
5. 自动化、浏览器、构建、合同/规格/INDEX 后正常提交；真实用户账号不用于模拟授予验收。

## Given/When/Then

- GIVEN 同一幂等请求并发或重试；WHEN 模拟成功/重置；THEN 同一结果最多一条订单，真实权益不变。
- GIVEN 测试码只剩一次额度；WHEN 两个账号并发兑换；THEN 最多一个成功，失败不写订单/权益，已用账号不能重复领用。
- GIVEN mock 关闭或码停用/过期；WHEN 直接 API 兑换或调整；THEN 拒绝，历史只读、真实 pro 不变。

## 验收记录

- [x] 后端隔离/事务/幂等/并发：全后端 121 项通过，新增模拟订单幂等/参数冲突、审计回滚、跨账号最后额度、重复/停用/过期码、真实路径隔离与历史只读。
- [x] 管理台和桌面/Web/native 入口：管理端 69、桌面 305、Web 33 项通过；追加两客户端 TEST 码按钮→模拟 Pro、不改真实 Pro 回归通过，native cargo check 成功（既有 unused 警告未改）。
- [x] 三端生产构建、210 份文档门禁通过；本地后端已更新，浏览器能查看真实与模拟权益、空模拟订单和测试批次表单。未给现有用户调整权益或生成批次。
- [ ] 正常 Git 提交仍需通过 WorkLog IDE 评审门禁，不绕过。
