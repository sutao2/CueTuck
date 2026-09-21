# beta.22 Skills 工具栏修复发行

状态：已发布并完成官网切换（2026-09-21）。

1. 统一版本为 0.1.0-beta.22，同一提交构建 Mac arm64 与 Windows x64，保持应用标识、数据目录和签名身份。
2. 完整远程回归、Windows 安装启动卸载与窗口控制验收、Mac DMG 和隔离更新安装验证通过后发布。
3. 上传七份附件并核对 SHA-256 与更新签名，再公开预览发行、切换官网入口；保留上一版官网以便回退。

- Given 本机 Skills 工具栏 When 用户升级 Then 得到已验收的下拉维护入口，详见 [修复计划](2026-09-21-skills-toolbar.md)。
- Given 任一构建或验收失败 When 准备发布 Then 修复并重新验证后才公开发行。
- Given 用户既有数据和后端 When 发布客户端 Then 不重置用户库或重新部署未改变的后端。

## 验收结果

- 冻结源码及发行标签 `abdb091c5caa2293aba1663a6fba138e72760cf2`；公开预览发行 [v0.1.0-beta.22](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.22)。七份远程附件 SHA-256 与本地一致。
- [完整远程回归](https://github.com/sutao2/CueTuck/actions/runs/35566846325)通过：三前端测试及构建、浏览器、后端、Mac 原生及 MCP。修复的本地界面验收见对应计划。
- [Windows 构建及安装验收](https://github.com/sutao2/CueTuck/actions/runs/35566846328)通过：135 项原生测试通过、6 项既有忽略；x64 NSIS 安装、启动、卸载和真实窗口最大化/还原/最小化/关闭通过。
- Mac DMG 完整性、只读挂载、版本、arm64、生产 API、二进制及固定证书验证通过。真实更新器的签名下载及临时隔离安装通过，未替换用户应用或真实库。
- 两平台更新签名通过，篡改字节被拒绝；匿名读取 latest.json 与本地一致，双平台安装包下载链接 HTTP 200，标签指向冻结源码。
- 官网切换至 `/opt/cuetuck-website-preview/releases/beta22-abdb091c`，公网 site.js 与本地哈希一致，官网 HTTP 200，Postgres/Redis/MinIO 健康。上一目录 `beta21-48a1391b` 保留用于回退，未部署后端或改写持久卷。
- 发行资产在 `output/releases/v0.1.0-beta.22/`；保持现有预览版的签名范围：Mac 固定证书签名但未经 Apple 公证，Windows 未配置 Authenticode。
