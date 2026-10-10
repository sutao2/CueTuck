# beta.28 恢复空输入搜索栏发行

## 计划

统一版本 0.1.0-beta.28，同一提交构建 Windows x64 / Mac arm64。复用固定代码及更新签名；完整 CI、Windows 安装启动卸载、Mac 镜像与签名通过后公开预览发行，验证公开更新与官网入口，保留旧静态目录回滚。

## 场景

- Given 空输入且无反馈 When 唤起或清空 Then 仅显示 64 高搜索栏；输入后搜索与填写仍为所选固定展开尺寸。
- Given beta.27 When 预览通道检查更新 Then 发现 beta.28；同版不提示，稳定通道排除 beta。
- Given 双平台产物 When 发布 Then 同源、哈希及签名匹配，验证通过才公开。

## 记录

已发布部署（2026-10-10）。

- 源提交 `00139bee373ed252f51b4cd78ca86abdfb6a58f6`；[beta.28 公开预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.28)，prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38039235914)与 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38039235932)成功。Windows 原生测试、安装、启动、窗口操作与卸载通过。
- Mac 镜像完整性、只读挂载内版本及固定签名通过；公开更新清单签名下载与临时目录隔离安装通过，未替换用户应用。
- 七份公开附件回读 SHA256 一致，双平台签名验证通过且篡改拒绝，安装包匿名 HTTP 200。
- 现行客户端逻辑确认 beta.26 / beta.27 发现 beta.28，同版不提示、稳定通道不选 beta。
- 官网部署至 `/opt/cuetuck-website-preview/releases/beta28-00139bee`，健康检查通过，公网 site.js 与本地构建一致；旧 `beta27-c7d10d1e` 保留回滚，后端与持久卷不变。
- 证据及资产目录 `output/releases/v0.1.0-beta.28/`。Windows 原生外观与 DPI 人工验收、Apple 公证和 Windows Authenticode 边界沿用前版。
