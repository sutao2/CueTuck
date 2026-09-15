# beta.14 双平台发行

状态：已发布并验收。

1. 统一桌面版本至 0.1.0-beta.14，纳入 beta.13 后的 macOS 中文复制、启动器键盘选择与下拉框边界修复。
2. 检查最近 CI 桌面测试失败；仅修复可复现的发行阻塞，补对应验收。推送固定发行提交，双平台使用同一源码与生产地址，沿用既有更新签名密钥。
3. 执行必要测试与全量远程 CI，构建 macOS arm64 和 Windows x64。验证版本、签名、篡改拒绝、安装包与源提交，再生成更新清单与校验和。
4. 完整产物先上传草稿，核验后公开 GitHub prerelease，并确认旧客户端可发现新版；仅提交源码和验收记录，不提交凭据或构建日志。

- Given beta.13 客户端 When 检查预览更新 Then 发现 beta.14 的对应平台签名资产。
- Given 任一构建或必要检查失败 When 准备发布 Then 不公开不完整发行。
- Given 两平台发行包 When 核对 Then 版本、源码提交、正式 API 和签名信任一致。

- Given 多语言用户首页取代旧里程碑文案 When 运行发行隔离测试 Then 校验真实下载入口与预览版说明，不强制恢复旧里程碑句子。

本机与最近 CI 均复现同一 README 旧文案断言失败；本次调整为核对当前发行入口及预览版边界，保留错误旧状态的否定断言。

## 发行验收

- 固定发行源码：`07f5b4e667b5bf9e2700701fb08a64a02e098325`。macOS arm64 与 Windows x64 均使用此提交、`0.1.0-beta.14` 和正式 API `https://prompt.likh.cn`。
- 本机桌面测试 94 个文件、586 项通过；文档检查通过。[全量 CI](https://github.com/sutao2/CueTuck/actions/runs/34920060639) 的前端、后端和 Rust 检查均通过。
- [Windows CI](https://github.com/sutao2/CueTuck/actions/runs/34920060732) 构建成功，完成安装、启动 10 秒、卸载验证；产物记录的源码与 SHA-256 一致。
- macOS 应用版本、arm64 架构、深度严格代码签名检查及 DMG 完整性验证通过。当前使用 ad-hoc 签名，未做 Apple 公证；Windows 安装包未做 Authenticode 签名。
- 两平台更新包均通过现有更新公钥验签，并拒绝篡改后的字节；更新清单、签名文件和 SHA256SUMS 一致。
- [公开发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.14) 为 prerelease，7 个附件的远端 SHA-256 与本机一致。匿名访问发行元数据和下载更新清单成功；公开发行列表按客户端预览渠道规则排序，beta.12 / beta.13 均可发现 beta.14。
- 本次发布客户端安装包与更新资产；未重新安装本机客户端，未变更服务端部署。
