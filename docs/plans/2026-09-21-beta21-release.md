# beta.21 Windows 一体式顶栏发行

状态：已发布并完成官网切换（2026-09-21）。

1. 统一版本为 0.1.0-beta.21；同一提交构建 Mac arm64 和 Windows x64，保留应用标识、数据目录和签名身份。
2. 完整 CI 通过；Windows 安装后实际调用顶栏最小化、最大化、还原和关闭按钮，并验证原生状态。Mac 核验包、固定证书和更新签名。
3. 七份附件上传草稿，核对哈希后公开预览版，切换官网下载入口并验证匿名下载、更新清单和业务健康；旧官网目录保留可回退。

- Given Windows 用户升级 When 打开应用 Then 使用一体式顶栏，窗口控制仍能操作原生窗口；细节见 [顶栏计划](2026-09-21-windows-titlebar.md)。
- Given 任一平台或窗口按钮验收失败 When 准备发布 Then 修正并重新验证，不发布不完整发行。
- Given 既有用户数据及后端 When 发布客户端 Then 不重置用户库、不重新部署未改变的后端。

仍为无 Apple 公证、无 Windows Authenticode 的预览版；CI 窗口操作不代表完整多显示器、DPI 和 Windows 11 贴靠浮层手工验收。

## 发行验收

- 冻结源码及标签 `48a1391b625f514059f93b494753fad463f32d78`，公开预览发行 [v0.1.0-beta.21](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.21)。七份附件的 GitHub SHA-256 全部与本地一致。
- [完整远程回归](https://github.com/sutao2/CueTuck/actions/runs/35556646703)通过：三前端测试/构建、API 合同检查、桌面及管理端浏览器、后端、Mac 原生与 MCP。
- [Windows 构建与原生界面验收](https://github.com/sutao2/CueTuck/actions/runs/35556646711)通过：135 项原生测试（6 项既有忽略）、x64 NSIS 安装、启动与卸载。UI Automation 实际调用最大化、还原、最小化和关闭按钮，并核验原生窗口状态；顶部几何检查确认无额外系统标题条。
- Mac DMG 完整性、只读挂载、版本、arm64、生产 API、二进制一致性及固定证书严格签名校验通过。真实更新器通过 localhost 重放签名包，完成下载进度和临时隔离应用安装，未替换用户应用或真实库。
- 两平台更新签名验证通过，篡改字节被拒绝；公开 latest.json 匿名读取与本地一致，双平台安装包匿名链接 HTTP 200，标签指向冻结源码。
- 官网已切换至 `/opt/cuetuck-website-preview/releases/beta21-48a1391b`，公网 site.js 与本地 SHA-256 一致；官网和管理端 HTTP 200，API 的 Postgres、Redis、MinIO 全部健康。旧目录 `beta20-b3e11a35` 保留用于回退，未重启或改写后端及持久卷。
- 本机包与验收文件在 `output/releases/v0.1.0-beta.21/`，不提交安装包、签名秘密或部署凭据。
