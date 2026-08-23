# 变更提案：M5 后端合同

| 字段 | 值 |
|---|---|
| 状态 | 接受 |
| 日期 | 2026-08-23 |
| 影响规格 | [square](../../specs/square/spec.md) · [auth](../../specs/auth/spec.md) · [publish](../../specs/publish/spec.md) · [test-gates](../../reference/test-gates.md) |

已接受。现行决定见 [ADR 0008](../../architecture/decisions/0008-m5-backend-contract.md)。尚未归档：OpenAPI 与规格测试映射未进现行文件。

## 为什么

[ADR 0003](../../architecture/decisions/0003-local-first-phase1.md) 把后端选择留到 M5。旧 Spring Boot 有认证、同步、广场、计费，但与两级分类、合集作内容、匿名下载不对齐。对接旧路径会把桌面拖进旧合同。

M5 进入标准还要求：覆盖率数字先写死。本提案写死数字；接受后写入测试门禁。

## 做什么

1. **改写 API，不复用旧端点。** 本仓库新增 OpenAPI 合同。旧 `PromptLauncher` 后端只读参考，不是运行时。
2. **账号：邮箱 + 密码。** 本服务签发 Access / Refresh。Refresh 进系统钥匙串，不进 Web Storage。不绑定 QQ / LinuxDo / Google / GitHub；OAuth 另开变更。
3. **广场权限与规格一致：** 浏览与下载不登录；收藏与发布要登录并写明原因。下载写入本地 `source=downloaded`。发布不锁、不删本地正文。
4. **离线：** 广场显示非阻断说明并提供回本地入口。本地库与启动器仍 100% 可用。启动器本地查询不得等网络。
5. **分类：** 广场条目映射到系统大分类；合集只出现在内容区。
6. **后端覆盖率（写死，未达不得合并后端）：** 新后端生产代码行覆盖率 ≥ 80%、分支覆盖率 ≥ 70%；API 用 Testcontainers；每个 OpenAPI path 有客户端映射测试。

## 不做什么

- 管理后台、计费、云同步、完整 Web 个人库（M6 或更后）
- 把启动器改成覆盖层，或让启动器搜广场
- 在提案被接受前创建 `backend/`、接 HTTP、打开 OAuth
- 宣传未上线的广场或未验证的商店分发

## 依赖

M4 已完成。本提案被接受后，才执行 [M5 计划](../../plans/2026-08-23-m5-online-square.md)。
