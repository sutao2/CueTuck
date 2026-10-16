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

验收中。沿用预览版固定 Mac 签名与 Windows 发布者签名边界；本次无服务端迁移。
