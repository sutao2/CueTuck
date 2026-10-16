# beta.35 更新角标发行

## 计划

1. 统一版本并固定源提交，构建 macOS Apple Silicon 与 Windows x64 包。
2. 完整 CI 和 Windows 安装、启动、窗口操作、卸载及免安装 ZIP 验收；Mac 验证固定签名、镜像与隔离更新安装。
3. 验证提示词、合集、社区 Skill 的只读提醒与明确确认保护，旧数据保持未知版本保护。
4. 所有门禁通过后发布 prerelease、签名清单与官网入口；核验匿名下载、旧版发现新版、同版不提示及稳定通道排除 beta。

## 场景

- Given 已修改的本地副本 When 检查新版 Then 仅显示角标，确认前不写入。
- Given 双平台包 When 发布 Then 源提交一致、签名及哈希校验通过。
- Given beta.34 When 检查更新 Then 能发现 beta.35；同版无更新。

## 状态

已于 2026-10-16 发布 beta.35。沿用预览版固定 Mac 签名与 Windows 发布者签名边界；本次无服务端迁移。

## 验收证据

- 同源提交 `d92c8ded27d3df346df95de02be4f20ac2d97cef`；[公开发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.35)，prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38858686973) 全部通过：桌面 726、Web 33、管理端 106 项测试及三个构建，客户端/管理台浏览器流程、后端、Mac 原生与 MCP。
- [Windows 验收](https://github.com/sutao2/CueTuck/actions/runs/38858686988) 通过：145 项原生测试（6 项显式忽略），同源 NSIS、ZIP 解压启动，安装/启动/真实窗口按钮/卸载。
- Mac DMG 镜像校验、既有固定证书、临时目录安装启动、旧库加载及真实版本比较通过；旧下载仍显示未知基准并禁止直接替换。运行前后本地提示词、合集、分类及成员记录指纹一致，临时应用已退出。
- 发布前本机清单与发布后公开清单均通过签名下载及隔离安装；双平台签名和篡改拒绝验证通过。
- 按现行客户端版本选择逻辑，beta.33 / beta.34 发现 beta.35，同版无更新，稳定通道排除 beta。
- 官网新目录 `/opt/cuetuck-website-preview/releases/beta35-d92c8de`；前版 `beta34-3574c75` 保留，可回退。数据库与 API 镜像未变。
- 证据在 `output/releases/v0.1.0-beta.35/`。新增角标与本地修改保护通过组件自动化回归；未宣称在 Windows 上人工逐页执行所有内容投稿和更新操作。


- 八份公开资产回读 SHA-256 一致；Mac DMG、Windows NSIS/portable ZIP、latest.json 匿名 HTTP 200，官网公网 site.js 与本地构建字节一致。
