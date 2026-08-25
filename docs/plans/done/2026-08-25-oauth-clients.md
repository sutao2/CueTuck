# 完成记录：客户端接 Google / GitHub

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-oauth-clients.md](../2026-08-25-oauth-clients.md) |

## 退出标准

- [x] 登录弹窗只列出 `GET /v1/session/oauth/providers` 返回的 Google / GitHub
- [x] 未配置则只留邮箱密码；不出现 QQ / LinuxDo
- [x] 桌面 Refresh 进钥匙串；web / admin-web 不把 Refresh 写入 Web Storage
- [x] 浏览器已登录收藏走 `PUT /v1/favorites/{id}`，不新增内存副本
- [x] 桌面 / web / admin-web 测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
89 passed

cd web && npm test
15 passed

cd admin-web && npm test
7 passed

./scripts/docs-check
docs-check 通过（138 个 Markdown 文件）。
```

未配置客户端凭据时提供商列表仍为空，不得发假请求。覆盖率 80%/70% 未实测。

## 文档

- 更新的规格：auth、web、admin
- 更新的 INDEX：是
- status.md：客户端 OAuth 已关闭；下一步见 deferred.md

## 未做 / 下一里程碑带走

- 云同步、账单、商店上架、自动更新安装
- Windows NSIS 与非 macOS 托盘仍未验证
- 未在真实 Google / GitHub 控制台凭据下做手工授权 smoke
