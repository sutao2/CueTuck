# beta.26 界面与语言优化发行

## 计划

1. 统一版本 0.1.0-beta.26，冻结同一源码构建 Mac arm64 和 Windows x64，复用现有签名身份与数据协议。
2. 完整 CI、Windows 原生测试/安装/启动/卸载与 Mac 包完整性通过后发布预览版，包含广场排版和加载批次优化、设置控件精修、九宫格铺满及完整界面语言切换。
3. 验证七份公开附件、双平台更新签名、旧版发现新版/同版不提示/稳定通道不选 beta，以及隔离更新安装。
4. 仅更新官网静态版本和下载入口，保留旧目录回滚，不改后端或用户应用数据。

## 场景

- Given beta.25 用户 When 预览通道检查更新 Then 可下载 beta.26 并看到新界面，原有库和设置保持。
- Given 中英文界面 When 打开设置与广场 Then 使用已验收的语言和布局改动，具体场景沿用各专项计划。
- Given 任一平台构建或签名失败 When 准备公开发行 Then 修复后重新验证，不发布不完整资产。

## 结果

已发布部署（2026-10-10）。

- 源码和发行标签：`fcb74fa4a49154d4247e73797cbf5bb4b89a4cdf`；[beta.26 公开预览版](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.26)，prerelease=true。
- [完整远程回归](https://github.com/sutao2/CueTuck/actions/runs/38020849667)与 [Windows 流水线](https://github.com/sutao2/CueTuck/actions/runs/38020849702)成功。Windows 原生测试、x64 安装、启动、窗口操作与卸载通过。
- Mac arm64 包及只读挂载的 DMG 中 beta.26 版本与固定签名验证通过，DMG 完整性通过。公开更新清单的真实签名下载和临时目录安装通过，未替换用户应用。
- 七份远程资产回读 SHA256 全部一致；双平台签名校验通过且篡改被拒绝；两平台安装包匿名访问均 HTTP 200。
- 公开 releases 列表结合现行客户端逻辑确认：beta.24 / beta.25 发现 beta.26，同版不提示，稳定通道不选 beta。
- 官网切换到 `/opt/cuetuck-website-preview/releases/beta26-fcb74fa4`；容器健康，公网 site.js 与本地构建一致。旧 `square-20261010-38d3df5` 保留回滚；后端及持久卷不变。
- 资产目录 `output/releases/v0.1.0-beta.26/`；本地发行门禁 9 项及官网数据回归 7 项通过。Apple 公证、Windows Authenticode 与前版状态相同，不新增系统信任声明。
