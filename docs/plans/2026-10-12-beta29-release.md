# beta.29 Skill 修复与便携版发行

## 计划

1. 统一版本 0.1.0-beta.29，固定同一源码提交构建 Windows x64 和 Mac arm64。
2. 验证完整 CI、Windows 安装与便携版启动、Mac 固定签名及镜像。
3. 发布预览版，附 NSIS、portable ZIP、DMG、签名更新资产与 SHA256SUMS。
4. 验证匿名更新发现、签名下载和隔离安装，更新官网入口并保留回滚目录。

## 场景

- Given 本地版本目录 Skill When 发布 Then 成功准备完整文件包，详情按钮保留间距。
- Given Windows 下载 When 选择免安装版 Then ZIP 可解压运行，数据位置和更新方式已说明。
- Given beta.28 When 检查预览更新 Then 发现 beta.29，同版不提示，稳定通道排除 beta。
- Given 双平台包 When 公开 Then 同源、签名及哈希验证通过，发行标记 prerelease。

## 记录

发布中。
