# Windows 预览安装包

状态：beta.2 Windows x64 安装包已构建、验收并公开发布。用户要求补充 Windows 包，授权沿用现有公开仓库与预览发行。

## 步骤

1. 修改已有 Windows GitHub Actions：生产 API 显式设为 `https://prompt.likh.cn`，运行现有 preview 门禁；构建 NSIS x64、关闭 updater artifact，不加入签名私钥或凭据。
2. 手动运行支持选择源 ref，首轮选择已发布 `v0.1.0-beta.1`；修复版由固定的推送 commit 构建，保证与 macOS 应用源码一致；工作流改进留在默认分支，不移动已有发行标签。若需应用修复则另立版本，不拿不同代码冒充旧标签。
3. 在隔离 Windows runner 跑原生测试、编译安装器，核对构建二进制的 PE 架构与生产 API；静默安装、启动进程存活检查、停止该临时应用并卸载。此 smoke 不冒充人工 UI、快捷键、SmartScreen 或完整第三方登录验收。
4. 下载成功运行的 artifact，核对构建提交和 SHA-256，上传现有预览 Release 并更新说明及校验文件。无 Windows 签名证书，明确 Authenticode/SmartScreen 边界。

## Given / When / Then

- GIVEN 工作流缺少生产地址；WHEN 发行预检；THEN 构建失败，不能发布指向 localhost 的包。
- GIVEN 发行标签已存在；WHEN 手动打 Windows 包；THEN checkout 该标签并记录解析出的 commit，不移动原标签。
- GIVEN NSIS 生成成功；WHEN 在临时 runner 验收；THEN 静默安装成功、应用进程存活、架构和 API 正确，验证后清理该临时安装。
- GIVEN artifact 校验与源提交匹配；WHEN 上传；THEN Release 增加 Windows x64 包，保留 macOS 包，下载与校验文件可用。

## 验收

- 首轮固定 v0.1.0-beta.1 的运行 [34747145742](https://github.com/sutao2/PromptArk/actions/runs/34747145742) 在 9 分 9 秒内通过 Windows 原生测试、NSIS 构建、x64/API 校验、静默安装、运行 10 秒、卸载和 artifact 上传。
- 用户随后报告原生参考图下载失败，系统代理修复见[修复记录](2026-09-13-reference-system-proxy.md)。首轮 Windows 资产未加入公开 Release；后续发布 beta.2，保留 beta.1 原标签和 macOS 包。

- 修复版运行 [34747692606](https://github.com/sutao2/PromptArk/actions/runs/34747692606) 最终 success；83 项 Windows 原生测试通过、2 项显式环境测试忽略。生产地址门禁、release 编译、PE x64 与内置 API 校验、静默安装、进程保持运行 10 秒、卸载和 artifact 上传均成功。
- build-info.json 的 source_commit 与 beta.2 标签一致，api=`https://prompt.likh.cn`、architecture=x64，authenticode=NotSigned；仍不声称人工 GUI、系统快捷键、SmartScreen 或完整 OAuth 已验收。
- 公开文件 `PromptArk_0.1.0-beta.2_x64-setup.exe` 为 4,970,259 字节，SHA-256 `2241bd0176a13fed6ebf621f3a8f0923b80d8a053a05cf15edaec6bf6cffb4b8`。本机下载 artifact、构建清单与 GitHub Release 返回摘要一致；SHA256SUMS 同时列出 Windows 与 macOS。
- 发布链接及固定源码版本见[系统代理修复记录](2026-09-13-reference-system-proxy.md)。未改动 beta.1 原标签或安装包，未发布自动更新清单。
