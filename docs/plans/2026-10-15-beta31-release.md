# beta.31 阅读、合集与 Skill 发布修复发行

## 计划

1. 统一 0.1.0-beta.31，将 Markdown 阅读、合集封面下载和本地成员展示、Skill 发布选择同步一并发布。
2. 同一源提交运行完整 CI、Windows 原生测试及安装/便携启动验证，构建 Mac 固定签名安装包并校验镜像。
3. 发布预览附件、更新清单和校验和，验证旧 beta 发现新版、同版无更新及稳定通道隔离；执行隔离更新安装。
4. 切换官网下载入口，保留旧站点回滚，记录结果与未实测边界。

## 场景

- Given 三处已提交修复 When 构建双平台 Then 两平台均包含全部修复。
- Given beta.30 When 检查更新 Then 发现 beta.31，同版不提示，稳定通道不选择 beta。
- Given 公开发行 When 下载 Then NSIS、portable ZIP、Mac DMG、签名更新包和 SHA256SUMS 可用且一致。

## 状态

已发布部署（2026-10-15）。

- 源提交 `ccc3a413023d67cc78ddda09d85a2acec235fb7c`，[beta.31 预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.31)，prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38669691441) 与 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38669691563) 全部成功。Windows 原生、便携解压启动、安装启动、窗口操作及卸载通过。
- Mac 镜像完整性、只读挂载版本、固定证书签名通过；双平台更新包签名校验通过并拒绝篡改。
- 八份公开附件回读与本地产物哈希一致；Mac DMG、Windows NSIS 与 portable ZIP 匿名下载 HTTP 200。
- 从服务器匿名读取公开 releases，用现行客户端逻辑验证 beta.29 / beta.30 发现 beta.31、同版不提示、稳定通道排除 beta。公开 Mac 更新清单下载、验签及临时目录隔离安装通过，未替换用户应用。
- 证据和产物保存在 `output/releases/v0.1.0-beta.31/`。此前浏览器和回归覆盖三处修复；新安装包内真实用户合集下载与双平台人工页面操作尚未实测。Mac 公证与 Windows 发布者签名边界不变。


- 官网现行目录 `/opt/cuetuck-website-preview/releases/beta31-ccc3a41`，容器健康，公网 site.js 与本地构建字节一致。旧 `beta30-d4f4f0a` 保留回滚，后端与持久卷未变。
