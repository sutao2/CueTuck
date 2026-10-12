# beta.29 Skill 修复与便携版发行

## 计划

1. 统一版本 0.1.0-beta.29，固定同一源码提交构建 Windows x64 和 Mac arm64。
2. 验证完整 CI、Windows 安装与便携版启动、Mac 固定签名及镜像。
3. 发布预览版，附 NSIS、portable ZIP、DMG、签名更新资产与 SHA256SUMS。
4. 验证匿名更新发现、签名下载和隔离安装，更新官网入口并保留回滚目录。

## 场景

- Given 本地版本目录 Skill When 发布 Then 成功准备完整文件包，详情按钮保留间距。
- Given Windows 下载 When 选择免安装版 Then ZIP 可解压运行，数据位置和更新方式已说明。
- Given beta.28 When 检查预览更新 Then 发现 beta.29，同版不提示，稳定通道排除 beta。
- Given 双平台包 When 公开 Then 同源、签名及哈希验证通过，发行标记 prerelease。

## 记录

已发布部署（2026-10-12）。

- 源提交 `cbc0009ee71059c40f08bbb8d4c5185331cf0a8b`；[beta.29 公开预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.29)，prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38221922162)与 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38221922410)成功；包含原生测试、portable ZIP 含空格目录解压启动与主窗口检查、NSIS 安装启动及卸载。
- Mac DMG 完整性、只读挂载内版本及既有固定签名验证通过；双平台更新签名验证通过且篡改拒绝。
- 八份公开附件回读 SHA256 一致，DMG、NSIS 与 portable ZIP 匿名 HTTP 200；公开清单签名下载及临时目录隔离安装通过，未替换用户应用。
- 本机出口 GitHub 匿名 API 额度用尽（403，remaining=0）；改用服务器独立出口匿名读取公开 releases 列表，以现行客户端逻辑验证 beta.27 / beta.28 发现 beta.29，同版不提示，稳定通道不选 beta。
- 官网部署至 `/opt/cuetuck-website-preview/releases/beta29-cbc0009`，容器健康检查通过，公网 site.js 与本地构建哈希一致；上一版 `beta28-00139bee` 保留回滚。后端和持久卷不变。
- 资产及证据位于 `output/releases/v0.1.0-beta.29/`。Windows 手工视觉/DPI 检查、Apple 公证与 Windows Authenticode 的未验证边界保持。

