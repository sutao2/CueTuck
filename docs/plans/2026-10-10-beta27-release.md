# beta.27 启动器固定尺寸与 Raycast 风格发行

## 计划

统一版本 0.1.0-beta.27，以同一提交构建 Windows x64 与 Mac arm64 预览包；保留签名身份、API 与数据协议。完整 CI 和 Windows 安装启动卸载通过，验证 Mac 包、双平台签名及公开更新发现后发布。官网静态入口更新并保留旧版本回滚。

## 验收场景

- Given beta.26 用户 When 预览通道检查更新 Then 发现 beta.27，同版不提示、稳定通道排除 beta。
- Given 启动器 When 空输入、搜索、填写、返回 Then 保持设置中所选固定尺寸，列表和快捷操作采用已验证的 Raycast 方向。
- Given 双平台包 When 上传与下载回读 Then 源提交、版本、哈希和更新签名一致；验证未完成不公开。

## 记录

准备中。Windows 原生外观与 DPI 人工验收不由自动构建替代。
