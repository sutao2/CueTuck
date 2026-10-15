# beta.31 阅读、合集与 Skill 发布修复发行

## 计划

1. 统一 0.1.0-beta.31，将 Markdown 阅读、合集封面下载和本地成员展示、Skill 发布选择同步一并发布。
2. 同一源提交运行完整 CI、Windows 原生测试及安装/便携启动验证，构建 Mac 固定签名安装包并校验镜像。
3. 发布预览附件、更新清单和校验和，验证旧 beta 发现新版、同版无更新及稳定通道隔离；执行隔离更新安装。
4. 切换官网下载入口，保留旧站点回滚，记录结果与未实测边界。

## 场景

- Given 三处已提交修复 When 构建双平台 Then 两平台均包含全部修复。
- Given beta.30 When 检查更新 Then 发现 beta.31，同版不提示，稳定通道不选择 beta。
- Given 公开发行 When 下载 Then NSIS、portable ZIP、Mac DMG、签名更新包和 SHA256SUMS 可用且一致。

## 状态

进行中，尚未发布。
