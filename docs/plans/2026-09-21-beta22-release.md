# beta.22 Skills 工具栏修复发行

状态：准备发行。

1. 统一版本为 0.1.0-beta.22，同一提交构建 Mac arm64 与 Windows x64，保持应用标识、数据目录和签名身份。
2. 完整远程回归、Windows 安装启动卸载与窗口控制验收、Mac DMG 和隔离更新安装验证通过后发布。
3. 上传七份附件并核对 SHA-256 与更新签名，再公开预览发行、切换官网入口；保留上一版官网以便回退。

- Given 本机 Skills 工具栏 When 用户升级 Then 得到已验收的下拉维护入口，详见 [修复计划](2026-09-21-skills-toolbar.md)。
- Given 任一构建或验收失败 When 准备发布 Then 修复并重新验证后才公开发行。
- Given 用户既有数据和后端 When 发布客户端 Then 不重置用户库或重新部署未改变的后端。

## 验收结果

待完成。保持现有预览版的签名范围：Mac 固定证书签名但未经 Apple 公证，Windows 未配置 Authenticode。
