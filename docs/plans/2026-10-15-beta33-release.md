# beta.33 合集预览与 AI 输出修复发行

## 计划

1. 统一版本 0.1.0-beta.33，发布本地合集封面大图和 AI 结构化正文校验。
2. 同源双平台构建、完整 CI、Windows 原生和安装/便携启动验证；Mac 固定签名及镜像检查。
3. 公开 beta 发行，核对附件哈希、更新发现、签名及隔离安装，切换官网并保留旧目录回滚。

## 场景

- Given 两处已提交修复 When 构建 Then 双平台均包含修复；不猜测封面与成员对应，不承诺 AI 语义质量。
- Given beta.32 When 检查更新 Then 发现 beta.33，同版不提示、稳定通道不选 beta。
- Given 公开发行 When 下载 Then 双平台安装包、Windows 便携包及签名更新资产完整可用。

## 状态

已发布部署（2026-10-15）。

- 最终源提交 `0a9b09384cf4a7d35ead3daa0099c30d087242f9`；[beta.33 发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.33)，prerelease=true。
- 修正后 [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38715374238) 和 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/38715374147) 全部通过。包含 Windows 原生、便携启动、安装启动、窗口操作和卸载检查。
- Mac 最终包镜像完整性、版本与固定证书签名通过；Windows 产物源提交与 Mac 一致，双平台更新签名通过并拒绝篡改。
- 证据与产物保存在 `output/releases/v0.1.0-beta.33/`。未调用用户实际 AI 供应商验证语义质量，双平台原生页面人工操作未实测；Mac 公证与 Windows 发布者签名边界不变。


## 首轮回归修正

完整前端回归发现封面组件新增查看器成为多根节点后，父级传入属性不再自动落到封面根元素。显式转发到封面，恢复卡片属性与事件，新增回归；双平台从修正后的同一提交重建。

## 公开验收

- 八份公开附件回读哈希一致，Mac DMG、Windows NSIS 与 portable ZIP 匿名 HTTP 200。
- 服务器匿名 releases 列表按现行客户端逻辑验证 beta.31 / beta.32 发现 beta.33，同版不提示、稳定通道排除 beta。公开清单签名下载与 Mac 临时目录隔离安装成功，未替换用户应用。
- 官网现行目录 `/opt/cuetuck-website-preview/releases/beta33-0a9b093`，容器健康，公网 site.js 与本地字节一致。旧 `beta32-f09ad76` 保留回滚，后端和持久卷不变。
