# beta.27 启动器固定尺寸与 Raycast 风格发行

## 计划

统一版本 0.1.0-beta.27，以同一提交构建 Windows x64 与 Mac arm64 预览包；保留签名身份、API 与数据协议。完整 CI 和 Windows 安装启动卸载通过，验证 Mac 包、双平台签名及公开更新发现后发布。官网静态入口更新并保留旧版本回滚。

## 验收场景

- Given beta.26 用户 When 预览通道检查更新 Then 发现 beta.27，同版不提示、稳定通道排除 beta。
- Given 启动器 When 空输入、搜索、填写、返回 Then 保持设置中所选固定尺寸，列表和快捷操作采用已验证的 Raycast 方向。
- Given 双平台包 When 上传与下载回读 Then 源提交、版本、哈希和更新签名一致；验证未完成不公开。

## 记录

已发布部署（2026-10-10）。

- 冻结源码 `c7d10d1ec363eb3e13fcfe258cb6977434679a96`，公开 [beta.27 预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.27)。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38032907174)与 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38032907138)全部成功，Windows 安装、启动、窗口操作与卸载通过。
- 首轮完整回归发现品牌图标测试绑定旧 CSS 类名；已改为核对三个图标的资产引用，并重新冻结源码和构建双平台包。
- Mac 固定签名、DMG 校验及只读挂载中的应用版本/签名通过；公开清单签名下载与临时目录隔离安装通过，未替换用户应用。
- 双平台更新签名通过，篡改拒绝；七份公开附件回读 SHA256 一致，Windows/Mac 安装包匿名 HTTP 200。
- 现行客户端选择逻辑确认 beta.25、beta.26 发现 beta.27；同版不提示；稳定通道排除 beta。
- 官网部署到 `/opt/cuetuck-website-preview/releases/beta27-c7d10d1e`，容器健康，公网 site.js 与本地构建一致；保留 `beta26-fcb74fa4` 回滚，未变更后端和持久卷。
- 资产及证据在 `output/releases/v0.1.0-beta.27/`。Windows 原生外观与 DPI 人工验收不由自动构建替代；Apple 公证/Windows Authenticode 边界沿用前版。
