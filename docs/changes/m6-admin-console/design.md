# 设计：M6 管理台

对照 [提案](proposal.md) 与 [ADR 0009](../../architecture/decisions/0009-m6-admin-console.md)，不重复「为什么」。尚未实现。

## 方案

- 合同：`docs/reference/openapi/admin.yaml`，前缀 `/v1/admin`。
- 服务：扩展本仓库 `backend/`（内存预发）。用户增加 `role`；无管理员角色的 Access 对写路径返回拒绝。
- 客户端：新建 `admin-web/`（Vue 3 + Vite + Vitest）。与 `desktop/` 无打包依赖。
- 会话：管理端只持 Access 于内存；关闭标签即失效。
- 审核：改远端 publication 状态；不回调桌面库。
- 设置：一项布尔开关，默认保持 M5 匿名浏览可用。

## 文件

| 路径 | 职责 |
|---|---|
| `docs/reference/openapi/admin.yaml` | 管理合同（接受提案后） |
| `backend/src` | `/v1/admin/*`、角色、审核、用户列表、开关 |
| `admin-web/` | 登录、审核、用户、设置页 |
| `desktop/` | 仅加「包内无 admin-web」断言；不嵌管理 UI |

## 风险

- 预发不是生产：不得把本机 `backend/` 写成已上线。
- 浏览器无钥匙串：管理会话比桌面更短命，须在规格里写清。
- 覆盖率门禁仍约束 `backend/` 生产代码；管理路径计入同一后端包。
