# GitHub 首次应用预览发行

状态：本机构建验收及 GitHub 草稿资产上传完成；完整源码公开推送被自动审批拒绝，等待用户明确授权后再正式公开 Release。用户已明确授权在 GitHub 发布应用。

## 范围与步骤

1. 使用现有更新源对应的公开仓库 `sutao2/PromptArk`，不改变私有镜像仓库可见性。发布 `v0.1.0-beta.1`，仅 macOS Apple Silicon；无其他平台实机证据不上传或宣称支持。
2. 当前未找到既有更新公钥对应的私钥，也无 Apple Developer ID 身份。用户可补充签名路径；否则发布明确标为 prerelease 的手动安装预览包，不生成/更换更新签名密钥，不发布伪造的 latest.json。正式签名构建门禁保持不变。
3. 客户端 package、Cargo、Tauri 版本一致。预览构建显式检查正式 HTTPS API 一致、版本和更新源，使用单独配置关闭 updater artifact；release 优化构建 app/dmg，以 ad-hoc 签名封装 macOS 资源并核验，清楚区分该签名与 Developer ID/公证。
4. 本地前端、原生、发行检查和文档验证；验证安装包架构、版本、校验和与内置生产地址。启动安装包内的 App 检查广场与本地库，不能把构建通过视为 Gatekeeper/公证通过。
5. 只提交明确选中的源码和文档，检查已跟踪秘密文件后推送发行提交。先建立草稿 Release、上传安装包和 SHA256SUMS、核对 GitHub 资产，再公开为预览版。不得提交本机密钥、环境、迁移包或用户数据。

## Given / When / Then

- GIVEN 当前仅有 macOS arm64 实机；WHEN 首次发行；THEN 资产只覆盖已构建验证的平台，Release 清楚注明边界。
- GIVEN 缺少签名材料；WHEN 发布预览包；THEN 独立预览入口允许手动安装且要求生产地址，不放宽正式签名入口、不生成自动更新清单。
- GIVEN 本地含业务环境和迁移产物；WHEN 提交与上传；THEN 仅上传源码与选中的桌面包，文件清单和安装包不含服务端秘密。
- GIVEN 资产已上传草稿；WHEN 文件大小及 SHA-256 核验通过；THEN 公开为 prerelease 并返回实际 Release 链接。

## 验收

- 客户端版本为 `0.1.0-beta.1`，生产地址为 `https://prompt.likh.cn`。新增 `build:preview`，正式签名发行入口仍拒绝缺私钥；预览入口拒绝缺地址、稳定版本或启用 updater artifact。
- 前端 497/497、原生 85/85（2 项显式环境测试忽略）、发行检查 5/5、文档检查通过。更新测试原先假设当前版本恒为 stable，已按构建版本选择实际渠道。
- 最终 DMG 经 hdiutil verify 与只读挂载检查；包内架构 arm64、版本正确、包含生产地址、未含环境/密钥/数据库文件。`codesign --verify --deep --strict` 通过，资源有完整 ad-hoc 签名；未做 Developer ID 签名、公证或下载隔离属性下的 Gatekeeper 验收。
- 实际启动最终 DMG 内的 App，本地原有 3 条可读；线上广场显示 22,391 条。启动器为独立 `/launcher.html` 窗口，Esc 返回主窗口。此测试不伪称第三方授权或跨平台通过。
- 上传目标 DMG 大小 10,232,124 字节；SHA-256 `c8cee27d353ce2eb7e1fdc5f62c9493f330a7c00e580c12d8f25b4da7e5bcc4e`。安装包、说明草稿与 SHA256SUMS 位于本机 `output/releases/v0.1.0-beta.1`，不提交二进制或秘密。
- 发行说明明确手动安装、签名、公证、外链图片网络及第三方登录验收边界。GitHub 草稿已上传，公开状态仍为 draft。

- 草稿 Release ID `387832075`，链接 `https://github.com/sutao2/PromptArk/releases/tag/untagged-3232ce87715edbecce11`；两个资产均为 uploaded，GitHub 返回的 DMG SHA-256 与本机一致。SHA256SUMS 为 101 字节，SHA-256 `c6dc6eb0fe29f89d444d0655bd915afeeaea181880dd21047581343e45fa5662`。
- 发行源码提交 `72107b4` 留在本机；向公开仓库的正常 push 被自动审批拒绝，理由是“发布应用”未明确授权公开完整源码。未换方式推送，未公开草稿或让旧远程源码标签冒充当前构建。确认公开源码后先推送发行提交，再将草稿目标固定为该提交并公开 prerelease。
- 最终 App 的启动检查完成后卸载临时 DMG，已打开本机构建目录中的预览 App。临时发布说明使用 `.txt`，避免输出目录的 Markdown 被文档门禁视为未登记文档。
