# 变更提案：完整产品队列

| 字段 | 值 |
|---|---|
| 状态 | 接受 |
| 日期 | 2026-08-25 |
| 影响规格 | [settings](../../specs/settings/spec.md) · [sync](../../specs/sync/spec.md) · [web](../../specs/web/spec.md) · [billing](../../specs/billing/spec.md) · [auth](../../specs/auth/spec.md) |

2026-08-25 人要求继续开发，不分第一期第二期，计划完成整个项目。视为接受。决定以 [ADR 0014](../../architecture/decisions/0014-full-product.md) 为准。

## 为什么

M9 之后只剩 `deferred.md` 里的另立项。设置十类里的云同步、更新、作者主页、我的发布、账单仍标明尚未提供。人要求把整个产品做完，而不是停在第一期本地。

## 做什么

1. 账号与广场：作者主页、我的发布、下载保留作者信息。
2. 个人库云同步：登录后桌面 SQLite 与预发库互相同步；浏览器登录后用同一账号库，不再假装已与桌面文件同步。
3. 自动更新：检查、通道、发行说明、可安装的自动下载；不声称已上架商店。
4. Windows / Linux 开机启动与托盘：写出行为，未验证平台不得勾选 QA。
5. 预发账单：订阅 / 兑换在预发 API 上真实可测；无支付密钥时不得写成已付费。

## 不做什么

- 原生移动端、QQ / LinuxDo、ChatGPT 插件、启动器改覆盖层
- 声称 Mac App Store / Microsoft Store 已上架
- 声称生产托管或公开下载
- 额度恢复前声称 Windows NSIS 已验证
- 删除已有本机设置

## 依赖

预发 Postgres 与客户端 OAuth 已关闭。
