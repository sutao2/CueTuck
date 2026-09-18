# beta.19 封面发布修复发行

状态：已发布，官网下载入口已更新（2026-09-18）。

## 步骤与验收

1. 统一版本为 0.1.0-beta.19，冻结同一源码构建 macOS arm64 与 Windows x64；保持应用标识、数据目录、固定 Mac 签名证书与更新密钥不变。
2. 完整 CI、Windows 安装启动卸载及 Mac 安装包完整性验证通过；核对源码、生产 API、双平台更新签名和 SHA-256。
3. 上传完整七份资产到草稿，核对后公开预览发行；更新官网入口并验证公开下载与更新清单。

- Given 旧版封面类型标错 When 新版再次发布 Then 从字节识别类型，保留原图片及九宫格顺序。
- Given 窄窗口的广场卡片 When 查看操作 Then 按钮文字不被挤断。
- Given 双平台构建或资产核验失败 When 准备发行 Then 不公开不完整包。
- Given 服务端已启用 AI 直接审核 When 升级客户端 Then 展示准确审核文案；本次不重复部署未变的后端。

本版仍为预览版，不宣称 Apple 公证或 Windows Authenticode 签名。


## 发行验收

- 冻结源码与标签：`6b47514be97a961e7b4b6e1e3d0b068e70c68922`；公开预览发行 [v0.1.0-beta.19](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.19)。七份附件的 GitHub SHA-256 全部与本地一致。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/35331604622)通过：客户端 654、管理端 106、网页端 33 项；后端 182 项单元测试（4 项既有忽略）及集成测试、Mac 原生 136 项（6 项既有忽略）、MCP 与双端浏览器流程通过。
- [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/35331604604)通过；x64 安装、启动 10 秒、卸载通过，build-info 的源码和生产 API 与发行一致。
- Mac arm64 DMG 完整性、只读挂载、版本、二进制一致性、固定签名证书指纹与严格签名校验通过；未替换用户本机安装。
- 两平台更新签名验证通过，篡改字节被拒绝；公开 latest.json 匿名下载并逐项一致，Mac/Windows 安装包匿名链接 HTTP 200。
- 官网目录 `/opt/cuetuck-website-preview/releases/beta19-6b47514b` 已切换，公网 site.js 与本地一致，旧目录保留可回滚；API 数据库、Redis、MinIO 健康。本次未重复部署已上线的 AI 审核后端。
