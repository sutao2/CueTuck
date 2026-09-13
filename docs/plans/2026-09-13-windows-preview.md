# Windows 预览安装包

状态：设计完成，待构建。用户要求补充 Windows 包，授权沿用现有公开仓库与预览发行。

## 步骤

1. 修改已有 Windows GitHub Actions：生产 API 显式设为 `https://prompt.likh.cn`，运行现有 preview 门禁；构建 NSIS x64、关闭 updater artifact，不加入签名私钥或凭据。
2. 手动运行支持选择源 ref，本次选择已发布 `v0.1.0-beta.1`，保证与 macOS 应用源码一致；工作流改进留在默认分支，不移动已有发行标签。若需应用修复则另立版本，不拿不同代码冒充旧标签。
3. 在隔离 Windows runner 跑原生测试、编译安装器，核对构建二进制的 PE 架构与生产 API；静默安装、启动进程存活检查、停止该临时应用并卸载。此 smoke 不冒充人工 UI、快捷键、SmartScreen 或完整第三方登录验收。
4. 下载成功运行的 artifact，核对构建提交和 SHA-256，上传现有预览 Release 并更新说明及校验文件。无 Windows 签名证书，明确 Authenticode/SmartScreen 边界。

## Given / When / Then

- GIVEN 工作流缺少生产地址；WHEN 发行预检；THEN 构建失败，不能发布指向 localhost 的包。
- GIVEN 发行标签已存在；WHEN 手动打 Windows 包；THEN checkout 该标签并记录解析出的 commit，不移动原标签。
- GIVEN NSIS 生成成功；WHEN 在临时 runner 验收；THEN 静默安装成功、应用进程存活、架构和 API 正确，验证后清理该临时安装。
- GIVEN artifact 校验与源提交匹配；WHEN 上传；THEN Release 增加 Windows x64 包，保留 macOS 包，下载与校验文件可用。

## 验收

待执行。
