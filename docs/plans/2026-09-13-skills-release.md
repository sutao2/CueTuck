# Skills 客户端 beta.8 发行

| 字段 | 值 |
|---|---|
| 状态 | 已公开发布（2026-09-14 北京时间） |
| 授权 | 用户要求「上线一下，并推送打包发版」 |
| 范围 | 已完成的 Skills 管理、来源扩充与分类；客户端双平台更新 |

## 发布计划

1. 核对与 beta.7 的差异及生产服务健康；本轮没有服务端或管理端代码/数据库变更，不重建无关服务。
2. 统一版本为 0.1.0-beta.8，执行发布预检和适用测试，提交并推送已验源码。
3. 本机构建 macOS arm64 预览包，GitHub Actions 构建 Windows x64 包，绑定同一提交；核验签名、哈希和安装启动。
4. 生成双平台更新清单，发布 GitHub prerelease，验证公开资产与更新发现，再记录结果。

## 验收场景

- Given 两平台包 When 发布 Then 都对应同一源码和生产 API，版本与签名清单一致。
- Given 已安装 beta.7 When 检查更新 Then 能发现 beta.8，并得到对应平台带签名的下载地址。
- Given 预览发行 When 用户下载 Then 明示 macOS 未公证、Windows 未签名及 Skill 跨客户端实际加载边界。

## 进展

- [x] 源码 `cbf9237`、版本与生产健康核对；无 backend/admin-web/deploy 变更。
- [x] macOS arm64 release 优化构建、codesign 严格验证、DMG 校验和、更新签名及篡改拒绝通过；发布包实际启动并显示 beta.8、15 个来源及分类。
- [x] Windows CI 34766819907 全部通过；NSIS 安装、启动 10 秒、卸载完成。源码与 API、SHA256、更新签名与篡改拒绝已核对。
- [x] beta.8 双平台预览发行已公开；7 个资产大小/SHA256、标签源码、匿名发现接口及公开更新清单均核验通过。

## 发行中验收修复

原始发布提交 `cbf9237` 的浏览器 CI 在应用交互前因 ANSI 彩色日志无法匹配 `Local:` 失败。修正验收脚本去除颜色后判定启动，并将图片原尺寸和批量分类选择器的交互匹配更新为现行控件；不改变客户端运行代码。客户端安装包仍以 `cbf9237` 为统一来源，后续仅测试/文档提交不改变打包内容。原生及后端 CI 已通过，浏览器复验继续运行。

本机原生回归 112 项通过、6 项显式忽略（含需网络的人工触发测试）。发布预检 5 项通过。macOS 发布包打开原有 10 条本地提示词正常；没有覆盖 `/Applications` 中已安装版本。构建日志 `/tmp/cuetuck-beta8-build.log`，测试 `/tmp/cuetuck-beta8-native-tests.log`。

浏览器复验已通过管理端全部流程；桌面执行到附件图片缩放时发现另一处旧「适应窗口」匹配，已改为当前「适应」按钮后继续复验。

后续复验完成图片和变量流程；批量整理的可搜索控件在浏览器中的名称为「目标分类」，已依据失败快照修正定位后重跑。

## 最终结果

- [公开发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.8)，预览通道；发布时间 2026-09-13 16:13:57 UTC（北京时间 2026-09-14）。
- 两平台安装包和 Git 标签均对应源码 `cbf9237129df6a2f92c74bae21d7630b6381c443`。后续 `ab6fc03`、`f366c35`、`ee30535` 仅修正验收脚本/文档，`git diff cbf9237 -- desktop backend admin-web deploy` 为空。
- [Windows 构建与安装验证](https://github.com/sutao2/CueTuck/actions/runs/34766819907) 全部通过。
- [最终全量回归](https://github.com/sutao2/CueTuck/actions/runs/34767679865) 全部通过：前端单元/构建、桌面与管理端浏览器全流程、后端、原生及 MCP。桌面单元测试为 544 项。
- 7 个公开附件（macOS DMG、更新归档及签名，Windows EXE 及签名，latest.json、SHA256SUMS）均与本地大小和 SHA256 一致；两平台签名都用产品公钥验证，并测试篡改字节被拒绝。
- 未登录 GET GitHub releases 接口返回 200 并包含非草稿 beta.8；公开下载 latest.json 与本地逐字节一致，包含 darwin-aarch64 / windows-x86_64，版本顺序 beta.8 > beta.7。
- 线上 health 的 postgres、redis、minio 均为 true，管理端 HTTP 200。本轮无服务端/管理端运行代码与数据变更，无需重新部署 Compose。
- 正式已安装目录未覆盖；本机实际运行验证使用打包输出中的 beta.8。未替用户执行更新安装，也不声称 macOS 公证、Windows Authenticode 或各智能体实际加载已认证。
