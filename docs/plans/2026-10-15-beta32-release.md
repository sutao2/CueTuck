# beta.32 Skill 许可选择发行

## 计划

1. 统一版本 0.1.0-beta.32，发布 Skill 许可下拉选择与简述，保留原有许可及公开确认。
2. 同源双平台构建、完整 CI、Windows 安装/便携启动验证、Mac 固定签名与镜像检查。
3. 公开预览发行，回读附件哈希，验证更新发现、签名与隔离安装；同步官网并保留旧站点回滚。

## 场景

- Given 已有或未声明许可的 Skill When 发布 Then 原许可保留，常用许可可选且带简述，变更许可后重新确认。
- Given beta.31 When 检查更新 Then 发现 beta.32；同版不提示，稳定通道不选 beta。
- Given 公开发行 When 下载 Then 双平台安装包、Windows 便携包与签名更新资产完整可用。

## 状态

已发布部署（2026-10-15）。

- 源提交 `f09ad765d5634356bbbc4b01194297756e94e848`；[beta.32 发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.32)，prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38672345361) 和 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38672345251) 全部成功。Windows 原生、便携启动、安装启动、窗口操作和卸载验证通过。
- Mac 镜像完整性、只读挂载内版本与固定签名通过；双平台更新包签名通过并拒绝篡改，Windows 产物源提交与 Mac 一致。
- 证据与产物保存在 `output/releases/v0.1.0-beta.32/`。许可选择功能回归已通过；双平台原生窗口人工操作未实测。Mac 公证和 Windows 发布者签名边界不变。


- 八份公开附件回读哈希一致；Mac DMG、Windows NSIS 和 portable ZIP 匿名下载 HTTP 200。
- 服务器匿名 releases 列表按现行客户端逻辑验证 beta.30 / beta.31 发现 beta.32，同版不提示、稳定通道排除 beta。公开 Mac 更新清单、签名下载和临时目录隔离安装通过，未替换用户应用。
- 官网现行目录 `/opt/cuetuck-website-preview/releases/beta32-f09ad76`，容器健康，公网 site.js 与本地构建字节一致；旧 `beta31-ccc3a41` 保留回滚，后端与持久卷不变。
