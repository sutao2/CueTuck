# beta.24 Windows 启动器边角修复发行

## 计划

1. 同一源码构建 Mac arm64 与 Windows x64，保留数据协议和签名身份。
2. 完整远程回归、Windows 安装/启动/窗口控制/卸载及 Mac 签名包验证通过后发布。
3. 保持 prerelease=true；核对旧版发现新版、同版不提示、稳定通道不选 beta，以及版本专属更新清单和七份附件。
4. 切换官网静态目录，保留旧版回滚，不部署后端。

## 场景

- Given Windows 用户 When 升级 Then 获得实色直角启动器外框，内部控件与业务交互不变；原生视觉边界见专项记录。
- Given 预览通道旧版 When 检查更新 Then 发现 beta.24，并从版本专属清单下载签名包。
- Given 任一验收失败 When 准备公开 Then 不公开不完整发行。

## 结果

已发布部署（2026-10-09）。

- 冻结源码及标签 `1471a247eaefcfc4c19274b44eb70cf804429cf3`；[公开预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.24)，保持 prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/37933958727)及 [Windows 流水线](https://github.com/sutao2/CueTuck/actions/runs/37933958748)均成功；原生测试、安装、启动、窗口按钮与卸载通过。Windows 黑角的最终像素外观仍需用户设备复验，不将窗口 smoke 等同完整视觉验收。
- Mac DMG 完整性、只读挂载、包内 beta.24 版本和固定签名通过；公开版本专属清单经真实更新器完成签名下载及临时目录安装，未替换用户应用。
- 七份远程附件回读 SHA256 与本地一致，双平台签名通过且篡改被拒绝；双平台匿名安装包下载 HTTP 200。
- 公开 releases 列表结合原样客户端筛选函数验证 beta.22 / beta.23 均发现 beta.24、同版不提示、稳定通道不选 beta。
- 官网切换到 `/opt/cuetuck-website-preview/releases/beta24-1471a247`，容器健康且公网 site.js 哈希一致。旧 `beta23-7f9080f5` 保留回滚；后端及持久卷未变更。
- 资产目录 `output/releases/v0.1.0-beta.24/`；Apple 公证与 Windows Authenticode 状态仍与前版一致，未新增系统信任声明。
