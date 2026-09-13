# PromptArk 服务器与管理端部署

状态：已部署验收（2026-09-13）。用户明确授权部署到 119.28.232.185，并同步修改客户端服务地址。

## 方案与步骤

1. 沿用服务器宝塔 Nginx 的 vhost/证书目录；API 使用 prompt.likh.cn，独立管理端使用 prompt-admin.likh.cn。管理端 /v1 同源代理 API，避免开放跨域通配规则。DNS、资源、已有站点检查后再添加新配置。
2. 新建 /opt/promptark 独立 Compose 项目，API、管理端、Postgres、Redis、私有 MinIO 各自独立；仅 API/管理端的 loopback 端口给宿主 Nginx 使用，不复用其他项目数据库和对象存储。构建材料不包含本机 .env、数据库、私钥、node_modules 或 target。
3. 首次 owner 使用用户选择的邮箱与随机强密码，凭据保存在权限 0600 的部署文件；不使用开发默认账号、不启用 schema reset。数据库、对象与加密密钥持久化。此为新实例，不自动迁移本机用户/会话/广场或第三方密钥。支付保持 mock，未配置 SMTP/OAuth 不假报可用。
4. 新域名申请独立 TLS 证书、配置自动续期；Nginx 校验后 reload，验证旧站点未受影响。网关按真实连接 IP 限流，不信任外部随意提供的转发头。记录镜像版本/摘要、运行方式及回退步骤。
5. 实际验证健康接口的三个依赖、匿名接口、管理员登录和受保护接口、HTTPS/跳转/管理端浏览器；客户端前端和原生地址都改为 https://prompt.likh.cn，重新构建并重启验证。
6. 对应部署说明与 INDEX 同步更新，检查通过后提交部署切片；秘密文件不入 Git。

## Given / When / Then

- GIVEN 现有其他站点及容器正在运行；WHEN 添加 PromptArk Compose 与两个 vhost；THEN 不覆盖已有配置、卷和监听端口，旧站点仍正常。
- GIVEN 新实例与随机 owner 密码；WHEN 启动并登录管理端；THEN Postgres/Redis/MinIO 健康、账号为 owner，默认开发账号无法登录，未认证管理接口拒绝访问。
- GIVEN 公开域名；WHEN 请求 HTTP、HTTPS、SPA 路径及 /v1；THEN HTTP 跳转 HTTPS、证书匹配、页面与 API 路由正确，私有依赖端口不暴露。
- GIVEN 客户端已重新构建；WHEN 打开客户端请求服务；THEN 前端及原生使用新的 HTTPS API origin。

## 回退

首次发布失败仅停本项目并移除本次新建 vhost，保留所有数据卷及秘密。后续发布保留上一镜像，不使用 down -v 或 reset schema。备份边界以恢复手册为准。

## 验收

- API 与管理端镜像 `20260913-72d35f9`（应用源码基于 72d35f9）构建并运行；API/admin/Postgres/Redis 健康，MinIO 桶探测成功，一次性初始化 exit=0。初始化容器曾触发 128 MiB 限制，调整为 256 MiB 后通过；常驻内存限制保持有界。
- HTTPS 两个域名证书有效，含两个 SAN，有效期至 2026-12-12；HTTP 均 301 跳转；SPA 路径 200。API 根路径保留接口服务的 404，健康检查路径为 /v1/health。证书 dry-run 续期成功，systemd timer 已启用。
- 真实公网验证：两个 origin 的 /v1/health 均三个依赖 true；owner 登录成功，未登录 /v1/admin/me 与默认开发账号均 401；验证 API 会话已撤销。
- 管理端 101 项测试通过；真实浏览器 owner 登录、内容审核空状态及系统状态三项可连接均已验，测试后退出。截图在本机 output/playwright/promptark-production-admin.png 和 promptark-production-health.png（不入 Git）。
- 客户端 desktop/web 本机两种构建环境与 Rust 构建环境已切换到 https://prompt.likh.cn；macOS debug App 构建、重启成功（进程 95208）。原生二进制与前端产物均包含新 origin；实际点击广场后服务器 /v1/square/catalog 和 /v1/square/browse 均 200，客户端正确显示新实例空广场，本地 3 条仍在。
- 原有 inknote.likh.cn 仍 200，已有 MySQL/MinIO 容器未改动；本项目仅 18787/15174 绑定 loopback。部署 .env 和持久化加密密钥均 0600，API UID 10001。
- 初始 owner 为用户指定邮箱，随机密码仅在受限本机 output/deployment/promptark-admin-credentials.txt 与部署秘密文件；未提交秘密。线上为新业务库，没有迁移本机数据、OAuth/SMTP/AI 配置，支付明确 mock。
- docs-check 和 git diff --check 通过。客户端为 macOS 调试包，不宣称完成生产签名、公证、自动更新或 Windows/Linux 验收。
