# 变更提案：M6 管理台

| 字段 | 值 |
|---|---|
| 状态 | 草稿 |
| 日期 | 2026-08-23 |
| 影响规格 | [admin](../../specs/admin/spec.md) · [publish](../../specs/publish/spec.md) · [auth](../../specs/auth/spec.md) |

未接受。Agent 不得自称接受。接受前不得创建 `admin-web/`，不得扩展管理 OpenAPI，不得实现审核写路径。

## 为什么

[ADR 0008](../../architecture/decisions/0008-m5-backend-contract.md) 把管理台排除在 M5 之外。M5 已能提交 `pending` 发布，但没有运营审核、用户查看或运行时开关。[M6](../../plans/milestones/m6.md) 要求独立管理端规格进 INDEX，且写路径须在生产或预发可用。

本仓库没有独立生产集群。本提案把本机 `backend/` 定为预发写路径，不声称生产或公开服务。旧 Spring / `admin-web` 只读参考，不是运行时合同。

## 做什么

1. **另开管理合同。** 新增 `docs/reference/openapi/admin.yaml`，不把管理路径静默写进已接受的 `square.yaml`。
2. **同一 Axum 服务，新前缀。** 管理路由挂 `/v1/admin/*`。普通用户 Access 不能改发布状态。
3. **独立 `admin-web/`。** Vue + Vite 浏览器应用。不打进桌面安装包，不进 Tauri 窗口。启动器不请求管理接口。
4. **管理会话。** 邮箱 + 密码，且账号须有管理员角色。Access 只留内存。Refresh 不进 Web Storage。不绑定 QQ / LinuxDo / Google。
5. **审核。** 列出 `pending`，通过或驳回。不锁、不改、不删作者本地 SQLite。
6. **用户。** 只读列表：邮箱与角色。不改密、不删号。
7. **运行时设置。** 先做一项已文档化开关（匿名广场是否开放）。关闭后本地库与启动器仍全部可用。

## 不做什么

- 声称生产已上线、商店分发或公开下载
- 把管理页嵌进桌面或改成启动器覆盖层
- 登录收藏 `PUT`、令牌轮换、密码 KDF、Postgres（M5 遗留，另开或后续）
- OAuth、计费、完整 Web 个人库
- 在提案被接受前写管理应用代码或管理 API

## 依赖

M5 已完成。本仓库 `backend/` 作为预发。本提案被接受后，才执行 [M6 计划](../../plans/2026-08-23-m6-admin-console.md)。
