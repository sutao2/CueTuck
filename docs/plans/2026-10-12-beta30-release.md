# beta.30 登录存储与恢复发行

## 计划

统一版本 0.1.0-beta.30，双平台同源构建。通过完整 CI、Windows 原生测试及便携/安装启动验证、Mac 固定签名与镜像校验后公开预览发行。核对附件、更新发现和隔离更新安装，切换官网并保留旧版回滚。同步修正便携说明中的登录凭据位置。

## 场景

- Given 旧版钥匙串登录 When 升级 Then 首次重新登录一次，后续从本机 auth/session.json 恢复，不读取登录钥匙串。
- Given 鉴权 401 或启动断网 When 续期或联网 Then 按需合并恢复，错误明确且不跨账号重放。
- Given 同源双平台包 When 发布 Then 包含 NSIS、portable ZIP、DMG、更新签名和校验和。
- Given beta.29 When 检查预览更新 Then 发现 beta.30，同版无更新，稳定通道不选 beta。

## 记录

首次 Windows 原生检查发现网络重试测试依赖 macOS 的快速拒绝连接行为；Windows 连接等待导致用例未触发应用重试或超过 5 秒。为两个故障注入用例显式设置 100ms 连接超时，保持生产超时与重试策略不变，然后重跑双平台同源构建。

浏览器回归旧脚本在测试登录完成后仍点击游客按钮，新订阅已实时切换到登录与昵称页。更新为完成昵称后点击已登录账号进入设置，保留发布列表验收；不改应用行为。

## 发布结果

已发布部署（2026-10-12）。

- 源提交 `d4f4f0ab359156b7e68478f95d59c4002ffe67ba`；[公开预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.30)，prerelease=true。
- 最终 [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38227644241) 与 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38228064099) 全部成功。Windows 原生、便携版解压启动、安装版启动与卸载通过；最终手动构建使用完整 SHA（此前短 SHA 检出失败，无产物发布）。
- Mac 镜像完整性、只读挂载内版本与既有固定签名通过；双平台更新签名通过且篡改拒绝。
- 八份公开附件回读哈希一致；Mac DMG、Windows NSIS 和 portable ZIP 匿名 HTTP 200。便携说明已核对本机登录配置位置。
- 服务器匿名读取 releases 列表，以现行客户端逻辑验证 beta.28 / beta.29 发现 beta.30，同版不提示，稳定通道不选 beta；公开更新清单签名下载与临时目录隔离安装成功，未替换用户应用。
- 官网现行目录 `/opt/cuetuck-website-preview/releases/beta30-d4f4f0a`，容器健康且公网 site.js 与本地构建哈希一致；旧 `beta29-cbc0009` 保留回滚。后端与持久卷不变。
- 更新说明明确首次重新登录，AI 凭据存储保持。证据与产物在 `output/releases/v0.1.0-beta.30/`。真实账号跨重启/网络切换体验、Mac 公证与 Windows 发布者签名仍不冒充已验。
