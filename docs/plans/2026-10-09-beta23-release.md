# beta.23 双平台视觉更新发行

## 计划

1. 冻结同一源码，统一版本 0.1.0-beta.23；包含 Windows 启动器、Mac 窗口按钮、整体视觉与图片预览更新。
2. 完整远程回归及 Windows 安装/启动/窗口控制/卸载通过；构建固定身份签名 Mac 包并验证更新资产。
3. 核对双平台哈希和更新签名后公开预览发行，切换官网静态版本，保留旧版可回退。

## 场景

- Given 已有用户数据 When 升级 Then 保留应用标识、数据目录、签名身份及业务接口。
- Given 构建或验证失败 When 准备公开 Then 修复并重验后再发布。
- Given 新官网版本 When 部署 Then 下载指向本次发行，旧静态目录保留，后端与持久卷不变。

## 结果

已发布部署（2026-10-09）。

- 冻结源码及标签：`7f9080f5b945c5f43747ec47a8f31d4443795d73`；[公开发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.23)。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/37916277802)与 [Windows 流水线](https://github.com/sutao2/CueTuck/actions/runs/37916277635)全部成功。Windows 原生测试、安装、启动、窗口按钮与卸载通过。
- Mac DMG 完整性、只读挂载、arm64、包内版本与固定证书检查通过。真实更新器分别通过本地清单及公开 latest 地址，签名下载后仅安装到临时目录，未替换用户应用。
- 双平台更新签名验证通过，篡改字节被拒绝；七份 GitHub 附件回读 SHA256 与本地完全一致；公开清单与本地一致。
- 官网切换到 `/opt/cuetuck-website-preview/releases/beta23-7f9080f5`，容器健康、公网 site.js 哈希一致。旧 `beta22-abdb091c` 保留；未部署后端或更改持久卷。
- GitHub prerelease 标记会让现有客户端 `/releases/latest/download/latest.json` 返回 404，因此本次关闭该元数据标记并设为 Latest；版本号和说明仍明确为 beta 预览版。公开真实更新器验收通过后确认此兼容修复。
- 发布资产：`output/releases/v0.1.0-beta.23/`。仍未完成 Apple 公证或 Windows Authenticode；浏览器视觉验收与真实系统原生窗口外观验收范围保持各专项记录所述，不将安装 smoke 等同全部视觉验收。
