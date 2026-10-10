# beta.25 Windows 启动器边界发行

## 计划

1. 统一版本 0.1.0-beta.25，冻结同一源码构建 Mac arm64 与 Windows x64，复用现有签名身份和数据协议。
2. 完整远程回归、Windows 原生测试及安装/启动/窗口控制/卸载、Mac 签名包验证通过后公开预览发行。
3. 验证七份附件、双平台更新签名、旧版发现新版、同版不提示、稳定通道不选 beta，以及版本专属更新清单的隔离安装。
4. 官网切换到新静态版本并保留旧目录回滚，不部署后端。

## 场景

- Given Windows 用户 When 升级 Then 获得浅灰面板、完整细边框与系统阴影配置；系统圆角和最终像素外观的验收边界见[专项记录](2026-10-10-launcher-windows-outline.md)。
- Given 预览通道旧版本 When 检查更新 Then 发现 beta.25，使用该版本专属清单及签名包。
- Given 构建或验证失败 When 准备发行 Then 不公开不完整资产。

## 结果

已发布部署（2026-10-10）。

- 冻结源码及标签 `d874f3c631686edae4a5fbfe876938295b83fd73`；[公开预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.25)，保持 prerelease=true。
- [完整远程回归](https://github.com/sutao2/CueTuck/actions/runs/38015715637)与 [Windows 流水线](https://github.com/sutao2/CueTuck/actions/runs/38015715684)均成功。Windows 原生测试、安装、启动、窗口按钮与卸载通过；原生阴影/圆角最终像素外观仍需用户设备复验。
- Mac DMG 完整性、只读挂载、arm64、包内版本及固定签名通过。真实更新器使用公开版本专属清单完成签名下载和临时目录安装，未替换用户应用。
- 七份远程附件回读 SHA256 全部与本地一致；双平台更新签名验证通过且篡改字节被拒绝；两个公开安装包匿名下载均 HTTP 200。
- 匿名公开 releases 列表结合现行客户端筛选函数验证：beta.23 / beta.24 发现 beta.25，同版不提示，稳定通道不选 beta。
- 官网切换到 `/opt/cuetuck-website-preview/releases/beta25-d874f3c6`，容器健康且公网 `site.js` 与本地完全一致；旧 `beta24-1471a247` 保留回滚，后端和持久卷未变更。
- 资产目录 `output/releases/v0.1.0-beta.25/`。Apple 公证及 Windows Authenticode 状态与前版相同，未新增系统信任声明。
