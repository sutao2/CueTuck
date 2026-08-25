# 完整产品怎么落地

| 字段 | 值 |
|---|---|
| 状态 | 目标 |
| 关联 | [提案](proposal.md) · [ADR 0014](../../architecture/decisions/0014-full-product.md) |

## 账号面

`GET /v1/me` 返回邮箱、显示名、简介。`PUT /v1/me` 改资料。`GET /v1/publications/mine` 列出当前账号投稿。投稿表补 `author_email`。下载时若本机开关打开，把作者写入本地副本展示字段，不覆盖正文。

## 个人库同步

账号库存在 Postgres，不写 Flyway 库 `pl`。

- `GET /v1/library/changes?since=` 拉变更。
- `PUT /v1/library/changes` 推变更。
- 每条：`id`、`kind`（prompt / collection / category / setting / asset）、`payload`、`updated_at`、`deleted_at`。
- 冲突默认 `updated_at` 较新者胜；设置可选保留本地或保留远端。
- 封面与图片走已有 MinIO。开启「仅 Wi-Fi 同步图片」且无法判定为 Wi-Fi 时跳过资源、仍同步正文。
- 桌面离线写 SQLite；上线后推拉。启动器与 MCP 仍只读本机 SQLite。
- 浏览器已登录后工作台读写同一账号库，说明改为账号库，不得写成已打开桌面那份文件。

## 自动更新

Tauri updater 对 GitHub Releases。检查更新是真请求。无发行物时说明没有可用更新，不得写成已从商店安装。

## 账单

预发 `GET /v1/billing/status` 与兑换码。Stripe 仅在配置了测试密钥时跳转 Checkout。未配置时接口明确未开通，不得把免费账号写成 Pro。

## 仍诚实

商店上架、生产托管、Windows NSIS（额度）、未验证平台的开机启动 QA，没有证据就不勾选。
