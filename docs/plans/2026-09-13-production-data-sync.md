# 本机业务数据同步到线上

状态：已实施并验收，外部授权和邮件实发另验。用户选择广场、图片及站点/OAuth/邮件配置；保留线上账号和管理员密码。

## 步骤与边界

1. 核对本机 backend-db-1 / promptark / public 与线上 promptark-postgres-1 / promptark / public。源有 22,391 条广场、67 条字典、122 条带图内容共 188 个来源图引用，media_objects=0；源 OAuth 来自 backend/.env，邮件为加密数据库配置。
2. 先给线上数据库及密钥做受限加密备份；导出只含 square_items、catalog、catalog_redirects 的一致性快照。向临时表加载，检查 ID 冲突和列结构；事务按 ID 追加，遇到不同正文拒绝覆盖。已有线上字典若与源不同须显式报错而非静默改写。已查明旧本机字典缺少新增 region 字段：仅在源缺此字段时沿用线上该字段，其余差异仍拒绝。不复制账号、会话、收藏、历史邮件队列或开发模拟订单。
3. 保留正文、来源/许可、下载计数、分类、排序、完整图片引用；源没有已上传对象，因此不伪称图片字节已存入线上 MinIO。抽查来源图访问，后续可独立做受许可约束的图片离线存储。
4. 站点、OAuth、SMTP 使用线上管理 API 的版本校验与审计保存。邮件密钥在本机内存解密、通过 HTTPS 交给线上 API 用线上密钥重新加密；不替换线上加密密钥、不输出明文。OAuth 回调改为 https://prompt.likh.cn/v1/session/oauth/callback，提供商控制台登记与真实授权是独立验收；不擅自发送测试邮件。
5. 检查全量行数/正文和引用摘要、分页排序、图片抽查、配置可读取/密钥可解密、管理员仍可登录，客户端刷新看到数据。记录结果并提交代码/文档，不提交迁移包、凭据或备份。

## Given / When / Then

- GIVEN 线上已有 owner；WHEN 导入广场与配置；THEN owner 口令、角色、已有账号记录不变，源账号与会话不迁移。
- GIVEN 相同 ID 内容不同；WHEN 事务导入；THEN 拒绝并回滚；同包重复导入不产生重复。
- GIVEN 图片保存在来源 URL 而非本机桶；WHEN 同步；THEN 完整保留引用并报告外链边界，不把引用计数称为上传文件数。
- GIVEN 源 SMTP 用本机密钥加密；WHEN 配置迁移；THEN 在内存解密并通过线上 API 重新加密，线上已有密钥不变，旧邮件队列不迁移不重发。

## 验收

- 线上迁移前数据库与原加密密钥已做 AES-256-CBC/PBKDF2 加密备份，保存于 `/opt/promptark/backups/before-data-sync-20260913`（0700）；解密口令独立位于 `/root/.promptark-backup-password`（0600）。迁移包在本机 `output/production-sync-20260913` 与线上 `/opt/promptark/migrations/20260913`，均不入 Git。
- `scripts/export-square-sync.mjs` 只读取明确的本机库，一致性导出三张白名单表，事务拒绝不同 ID 内容冲突。真实 PostgreSQL 的字面量转义、字典兼容、重复导入、冲突及缺分类回滚测试通过；`PROMPTARK_LOCAL_SYNC_TEST=1 node --test scripts/export-square-sync.test.mjs` 为 2/2，通过默认测试时跳过需本机 Docker 的集成项。
- 全包回滚演练后正式 COMMIT 成功；同包再次执行新增 0 条。迁移包 SHA-256 为 `800a4dd994b0fdfdabc5f313e9a2f06c659635eff7e5b4978326fb2d426c7b8b`。
- 线上广场 22,391 条、字典 67 条；按 ID 排序的全量行摘要本机/线上均为 `5ac788c0c11f479276e4e1813eaac7bf`，覆盖正文、来源、下载计数和排序字段。122 条内容的 188 个图片引用完整保留，未把外链图复制进 MinIO。
- 前两页返回 96 个不同 ID；推荐、最新、热门接口正常，热门下载量降序已验。桌面客户端刷新后显示 22,391 条以及分类数量，本地 3 条保持不变。
- 三张来源图通过本机现有代理网络 HEAD 返回 200/image；服务器抽查图源也返回 200。本机直接连接图源超时，桌面截图仍出现图片占位；图片可见性受客户端网络影响，未声称直连图片展示通过。
- 站点字段逐项一致；SMTP/Google/GitHub 密文在内存解密与源凭据相等，线上原加密密钥与迁移前备份一致。原 owner 密码登录通过，线上仍只有原有 1 个账号；未迁移源账号、会话、收藏、邮件队列或模拟订单。
- OAuth 登录入口生成 Google/GitHub 授权地址及生产 HTTPS 回调。提供商控制台登记和真实授权未验；未发送测试邮件。Postgres、Redis、MinIO 健康检查均通过。
