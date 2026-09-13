# 部署与本地验收模板

此目录提供模板，不会自动安装服务、创建账号或替换现有数据库。

## 无外部凭据的集中验收

先安装 Node 22+、Rust、Python 3，分别在 `desktop`、`admin-web`、`web` 运行 `npm ci`。本机后端测试默认使用开发 PostgreSQL；连接与隔离恢复边界见 [本地开发](../docs/how-to/local-dev.md) 和 [恢复说明](../docs/how-to/backend-recovery.md)。

```sh
node scripts/verify.mjs
```

该入口跑三前端测试/构建、脚本测试、文档、后端、原生 Rust、MCP 及浏览器核心操作，任一步失败最终非零退出。日志在 `output/verification/run-*`，浏览器快照/截图在 `output/playwright/smoke-*` 和 `output/playwright/admin-*`，请勿提交。可传 `frontend`、`backend`、`native`、`mcp`、`browser` 单独复验。浏览器使用临时会话和专用 1431（客户端）、1432（管理端）端口；占用时拒绝，不连接用户现有服务。首次运行先在 `desktop` 执行 `npx playwright-cli install-browser chrome`。

管理端浏览器脚本 `node scripts/admin-browser-smoke.mjs` 使用隔离 API 替身，验证登录、列表返回、审核提交、配置保存失败/重试和退出；不代替真实后端或外部提供商联调。集中入口将后端测试并发限制为 4 个线程，以减轻本机开发数据库资源压力；测试内部的并发冲突场景仍然执行。

真实业务链路另在 `backend` 显式执行 `cargo test --locked real_business_browser_roundtrip -- --ignored --nocapture`，要求本机 PostgreSQL、已配置私有 MinIO、三前端依赖及 Playwright Chrome。使用随机 schema、临时 1433/1434 前端与动态 API 端口，完成后只清理本次数据。不会发外部邮件或操作用户桌面库；浏览器与原生覆盖边界见[业务验收记录](../docs/plans/2026-09-08-business-acceptance.md)。未配置 MinIO 的通用 CI 不自动运行此显式测试。

`.github/workflows/verify.yml` 使用 CI 临时数据库和独立浏览器；提交工作流不等于远程 CI 已通过。数据库恢复/真实 MinIO 与 1 万条性能演练是显式本机命令，不在通用 CI 上假报覆盖。

## 服务地址

三前端共同读取构建变量 `VITE_API_BASE`；原生 Rust 读取 `PROMPTARK_API_BASE`（启动环境优先，其次构建期值）。开发默认 loopback 8787。地址是 origin，不支持路径前缀、用户名、密码、query 或 fragment；非 loopback 必须 HTTPS。MCP 的显式广场地址仍为独立隐私边界，不自动继承已登录桌面配置。

当前本机使用 8080：后端从 `backend/.env` 读取绑定地址与 OAuth 回调；三前端的 `.env.development.local` / `.env.production.local` 仅放公开的 `VITE_API_BASE`，原生构建从 `desktop/src-tauri/.cargo/config.toml` 的 `PROMPTARK_API_BASE` 读取同一 origin。这些本机文件被 Git 忽略，不能把后端含密钥的 `.env` 复制给前端。修改后重启 Vite，并重新构建/打开客户端。仓库未配置时仍采用 8787 默认值。

正式构建前由部署方安全设置两个一致的 HTTPS origin。客户端不能携带服务端密码、签名私钥或 OAuth secret。原生 CSP 保持不允许任意 WebView 直连：令牌请求仍经 Rust 命令。

## 调试与发行分离

```sh
cd desktop
npm run build:local
```

本机命令输出无自动更新签名产物的 macOS debug App，不修改正式更新配置。正式入口为 `npm run build:release`，会检查版本、服务地址、更新源一致及签名材料存在；没有配套私钥拒绝构建。预检不等于签名验证：仍需核对公私钥匹配、托管 `latest.json`/签名/下载文件一致、升级安装与回退、macOS 签名公证，以及目标 OS 实机 smoke。既有更新仓库仅作配置一致性检查，不声称远端已经托管可用产物。

## 独立开发依赖（可选）

已有本机服务无需另起。需要全新隔离依赖时，先安全设置 `PROMPTARK_DEV_DB_PASSWORD`、`PROMPTARK_DEV_MEDIA_USER`、`PROMPTARK_DEV_MEDIA_PASSWORD`，再运行：

```sh
docker compose -f deploy/local-services.yml up -d
```

使用独立项目 `promptark-dev`、15432/16379/19000 端口及独立卷，不复用已有容器。自行创建私有 MinIO bucket 并配置后端 `PROMPTARK_MEDIA_BUCKET`；后端变量名称见 [环境模板](../backend/.env.example)。别对有数据的卷使用 `down -v`。此 compose 仅限开发，未经版本锁定、TLS、限流、备份和权限验收不得用作生产部署。

## Linux 服务模板（未在本机安装）

由运维创建 `promptark` 服务用户和 `/var/lib/promptark`，部署编译好的后端到 `/opt/promptark/promptark-api`。把权限为 0600 的环境文件放在 `/etc/promptark/api.env`，明确数据库、Redis、私有对象存储、固定密钥路径和 API 绑定地址（推荐 loopback + TLS 反向代理）。首次 owner 显式初始化，不启用开发默认账号，不打开 schema reset。对照 `promptark-api.service` 校验权限后再安装，支付继续 mock，禁用未配置的外部投递。

## 备份与切换

数据库、固定加密密钥、对象存储和部署配置是同一备份单元；完整操作及随机临时库恢复演练以 [恢复说明](../docs/how-to/backend-recovery.md) 为准，不复制第二套步骤。生产恢复必须明确授权目标并有回退点。本轮没有安装服务、切换生产或操作真实用户资料。
