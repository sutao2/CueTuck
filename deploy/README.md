# 部署与本地验收模板

`production/` 已在 119.28.232.185 部署独立 Compose 实例；其他开发与 systemd 文件仍为模板。当前验收见[部署记录](../docs/plans/2026-09-13-compose-deployment.md)。

## 当前 Compose 部署

- API：`https://prompt.likh.cn`；健康检查 `/v1/health`。
- 管理端：`https://prompt-admin.likh.cn`，静态页面由 Nginx 容器提供，`/v1` 同源转发 API。
- 服务器目录：`/opt/promptark/source/deploy/production`。宿主宝塔 Nginx 的独立配置为 `/www/server/panel/vhost/nginx/promptark.conf`，不覆盖其他站点。
- Postgres、Redis、MinIO 只在独立 Docker 网络；API/admin 分别只映射 `127.0.0.1:18787` / `127.0.0.1:15174`。持久卷为 `promptark_postgres`、`promptark_media`、`promptark_keys`。依赖与构建基础镜像固定摘要，应用镜像使用不可复用的发布标签。
- `.env` 由运维安全生成并设为 0600，变量见 `production/.env.example`；不能用模板占位值直接启动，也不能提交实际密码。首次创建 owner，关闭开发账号和 schema reset；后续修改环境中的初始化密码不会重置账号。

在服务器目录执行：

```sh
./compose ps
./compose logs --tail 100 api
./compose up -d --no-build --wait
```

重新部署时只上传明确筛选的源码，排除本机 `.env`、密钥、数据库、node_modules 和 target；根 `.dockerignore` 提供额外保护。为新版本指定新的 `PROMPTARK_IMAGE_TAG`，再 `./compose build`、`./compose up -d --no-build --wait`。首次模板启动前须将数据库、对象与密钥配置好。不要执行 `down -v`；保留上一版本镜像，数据库变更回退按恢复手册处理。

TLS 证书含两个域名，ACME 数据在 `/opt/promptark/letsencrypt`，宿主证书目录通过符号链接读取。`promptark-cert-renew.timer` 每天两次运行 `production/renew-cert.sh`，成功后校验并 reload Nginx；用 `systemctl status promptark-cert-renew.timer` 和 `journalctl -u promptark-cert-renew.service` 检查。自动续期的容器必须继续挂载 `/www/wwwroot/promptark-acme`，不能清理此验证目录。

此实例首次以空业务库上线，后按用户指定范围完成广场、图片引用、站点、Google/GitHub 和 SMTP 配置迁移，核验与图片网络边界见[数据同步记录](../docs/plans/2026-09-13-production-data-sync.md)。线上 owner 和加密密钥保留，本机账号、会话、收藏、历史邮件队列不迁移。OAuth 提供商还需登记正式 API 回调 `https://prompt.likh.cn/v1/session/oauth/callback` 并验证真实授权；SMTP 未做实发测试，AI 服务未在本次配置。支付保持 mock。备份须同时覆盖数据库、对象、固定加密密钥及秘密部署配置，详见[恢复说明](../docs/how-to/backend-recovery.md)。

## Skill 社区发布

客户端现在支持本机创建、完整包投稿及社区安装。管理端“Skill 社区审核”支持逐文件审阅、通过、退回和下架，投稿经人工审核后公开。当前镜像、请求体边界、数据库备份和平台验收范围见 [Skills 创建与发布记录](../docs/plans/2026-09-15-skill-publishing.md)。

## 官网与 Web 公开浏览

`https://prompt.likh.cn/` 为正式官网入口，旧 `/preview/` 跳转至根路径并保留页内导航。官网以独立启动器、全局快捷键、搜索和填写复制为主线；网页工作台从 `/#app`、`/#skills` 进入。提示词读取同源公开 `/v1/square/` API，24 条分页，计数来自服务端；Skills 使用与客户端相同的 15 个来源和分类规则，通过公开 GitHub API 固定提交并读取真实 SKILL.md。账号登录、云端收藏和私有数据尚未接入，暂存和草稿仅当前标签页有效。正式 `web/` 业务应用未被替换；数据范围见[真实广场与重设计](../docs/plans/2026-09-15-website-live-data.md)，官网与路由验收见[启动器官网](../docs/plans/2026-09-15-launcher-website.md)。

执行 `python3 scripts/build-website-preview.py`，构建白名单包括 HTML/CSS/JS、数据模块、品牌图标与真实桌面截图，并从客户端生成来源目录、分类模块和变量解析模块；不上传整个 `output/`。独立 Compose 模板在 `deploy/website-preview/`，部署目录 `/opt/cuetuck-website-preview/current`，只映射 `127.0.0.1:15175`。宿主 API vhost 通过可选 include 精确接管根地址和静态资源路径；既有 API catch-all、`/v1/` 和管理端路由保持原配置。CSP 只允许同源、GitHub API/原文和审核过的参考图域名；ES 模块 `.mjs` 显式返回 JavaScript MIME。

当前官网版本目录为 `/opt/cuetuck-website-preview/releases/20260915-beta16-1`（beta.16 下载入口），上一版 `20260915-launcher-1` 保留。根路由切换前的配置备份在 `/opt/promptark/backups/website-root-20260915`。回退静态版本时，对旧目录的 `compose.yml` 执行 `docker compose -f … up -d --wait`，健康检查后调整 `current` 链接。若同时撤回根路由，恢复该备份中的 `host-location.conf` 并检查、reload Nginx；数据库无需变更。首次路由配置备份为 `/opt/promptark/backups/website-preview-20260915/promptark.conf`，只有完全撤销预览时才核对后续配置后恢复、检查并 reload Nginx，再停止预览 Compose。不要停止 `promptark` 主业务项目或删除持久卷。

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

本机开发后端仍使用 8080：后端从 `backend/.env` 读取绑定地址与 OAuth 回调；客户端 desktop/web 的 `.env.development.local` / `.env.production.local` 已切换至 `https://prompt.likh.cn`；原生构建的 `desktop/src-tauri/.cargo/config.toml` 使用相同 `PROMPTARK_API_BASE`。本机管理台仍可连接开发服务，正式管理端通过 Docker 构建变量使用自己的同源代理。这些本机文件被 Git 忽略，不能把后端含密钥的 `.env` 复制给前端。修改后重启 Vite，并重新构建/打开客户端。仓库未配置时仍采用 8787 默认值。

正式构建前由部署方安全设置两个一致的 HTTPS origin。客户端不能携带服务端密码、签名私钥或 OAuth secret。原生 CSP 保持不允许任意 WebView 直连：令牌请求仍经 Rust 命令。

## 调试与发行分离

```sh
cd desktop
npm run build:local
```

本机命令输出无自动更新签名产物的 macOS debug App，不修改正式更新配置。正式入口为 `npm run build:release`，会检查版本、服务地址、更新源一致及签名材料存在；没有配套私钥拒绝构建。预检不等于签名验证：仍需核对公私钥匹配、托管 `latest.json`/签名/下载文件一致、升级安装与回退、macOS 签名公证，以及目标 OS 实机 smoke。既有更新仓库仅作配置一致性检查，不声称远端已经托管可用产物。

## 手动安装预览版

用户授权的首次 GitHub 预览发行见[发行记录](../docs/plans/2026-09-13-github-release.md)。在 `desktop` 执行：

```sh
TAURI_SIGNING_PRIVATE_KEY=/安全位置/cuetuck-updater.key PROMPTARK_API_BASE=https://prompt.likh.cn VITE_API_BASE=https://prompt.likh.cn npm run build:preview
```

此入口要求预发行版本、相同的正式 HTTPS API 与更新签名密钥。输出 release 优化并以 ad-hoc 签名封装资源的 macOS app/dmg，同时生成更新归档和签名；发行时还需生成 `latest.json`，不代表 Apple 签名/公证通过。GitHub 必须标为 prerelease，说明已验平台及限制。正式签名发行仍使用原 `build:release` 门禁。

### macOS 首次打开提示「已损坏」或无法验证

预览版的 ad-hoc 签名能校验包完整性，但不能替代 Developer ID 与 Apple 公证；浏览器下载后仍可能被 Gatekeeper 拦截（[Tauri 说明](https://v2.tauri.app/distribute/sign/macos/#ad-hoc-signing)）。

先确认下载来自本仓库 Releases，并核对 DMG 的 SHA-256 与发行附件 `SHA256SUMS` 一致。拖入 Applications 后，在终端检查：

```sh
codesign --verify --deep --strict --verbose=2 /Applications/CueTuck.app
```

校验失败应重新下载，不要重新签名掩盖损坏。校验成功且确认来源可信时，可按 [Apple 指引](https://support.apple.com/102445) 在「系统设置 → 隐私与安全」选择「仍要打开」。若显示「已损坏」且没有该入口，可在终端仅移除此应用的下载隔离标记，再自行打开：

```sh
xattr -dr com.apple.quarantine /Applications/CueTuck.app
```

此操作只放行指定应用，不关闭全局 Gatekeeper，不删除应用数据；只在确认来源和完整性后使用。后续替换新版本可能需要再次放行。无证书发行不保证首次无提示启动。

Windows x64 预览包使用现有 `desktop-windows` 工作流，在手动运行的 `source_ref` 中填写已发布标签或确切提交；留空则使用工作流提交。流程固定生产 API，执行原生测试、NSIS 构建、临时安装/启动/卸载，输出安装包、校验和与源提交记录。只上传与发行标签匹配的成功产物；Windows 未配置 Authenticode 签名。当前结果见[Windows 预览验收](../docs/plans/2026-09-13-windows-preview.md)。

## 独立开发依赖（可选）

已有本机服务无需另起。需要全新隔离依赖时，先安全设置 `PROMPTARK_DEV_DB_PASSWORD`、`PROMPTARK_DEV_MEDIA_USER`、`PROMPTARK_DEV_MEDIA_PASSWORD`，再运行：

```sh
docker compose -f deploy/local-services.yml up -d
```

使用独立项目 `promptark-dev`、15432/16379/19000 端口及独立卷，不复用已有容器。自行创建私有 MinIO bucket 并配置后端 `PROMPTARK_MEDIA_BUCKET`；后端变量名称见 [环境模板](../backend/.env.example)。别对有数据的卷使用 `down -v`。此 compose 仅限开发，未经版本锁定、TLS、限流、备份和权限验收不得用作生产部署。

## Linux 服务模板（未在本机安装）

由运维创建 `promptark` 服务用户和 `/var/lib/promptark`，部署编译好的后端到 `/opt/promptark/promptark-api`。把权限为 0600 的环境文件放在 `/etc/promptark/api.env`，明确数据库、Redis、私有对象存储、固定密钥路径和 API 绑定地址（推荐 loopback + TLS 反向代理）。首次 owner 显式初始化，不启用开发默认账号，不打开 schema reset。对照 `promptark-api.service` 校验权限后再安装，支付继续 mock，禁用未配置的外部投递。

## 备份与切换

数据库、固定加密密钥、对象存储和部署配置是同一备份单元；完整操作及随机临时库恢复演练以 [恢复说明](../docs/how-to/backend-recovery.md) 为准，不复制第二套步骤。生产恢复必须明确授权目标并有回退点。当前 Compose 为新建实例部署，未替换已有业务数据库；恢复或迁移已有数据需明确核对目标。

## CueTuck 签名更新

公开仓库为 [sutao2/CueTuck](https://github.com/sutao2/CueTuck)。首次 CueTuck 安装包为更新信任引导版本，旧预览包仍需手动安装一次；后续由同一私钥签名，通过每个发行的 `latest.json` 提供平台资产。私钥保存在发布主机受限文件和仓库 Actions Secret，不提交；预览及正式工作流必须携带 `TAURI_SIGNING_PRIVATE_KEY`，均生成更新签名资产。签名更新不等于 Apple 公证或 Windows 发布者签名。数据库文件、应用 identifier、凭据服务与线上持久卷继续使用旧内部标识以保持兼容，产品界面与新安装包使用 CueTuck。
