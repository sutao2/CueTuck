# Windows 启动器黑角修复

## 计划

用户 beta.23 截图显示启动器 CSS 圆角外侧为黑色，不能再以浏览器透明背景证明 Windows WebView 合成正确。采用不依赖透明圆角的 Windows 外观：窗口内容完整铺满为主题底色、外框直角，内部控件保留圆角。Mac 保持现状；不改变窗口大小、焦点或业务交互。

1. 仅 Windows launcher 根容器和画布使用实色，stage 去除外部圆角。
2. 验证 Windows/Mac 分支与启动器搜索、填写、AI 草稿回归及构建。
3. 记录 Windows 原生截图验收边界并提交；本轮不自动发布。

## 场景

- Given Windows 启动器 When 空搜索、结果列表或草稿显示 Then 主题背景铺满窗口，外框无透明圆角露出的黑色三角，内部操作仍可用。
- Given Mac 启动器 When 显示 Then 保留原有透明画布与圆角外框。
- Given 明暗主题和 64px/420px 窗口 When 布局切换 Then 内容不横向溢出，底栏可见，窗口尺寸逻辑不变。

## 验收

46 项启动器测试及桌面构建通过。Playwright 隔离页面模拟 Windows 平台，620×64 折叠与 620×420 展开均保持视口高度；展开画布实色、stage 圆角为 0、无横向溢出，浅深色已截图检查。Mac 分支仍为 18px 圆角和透明画布。

截图：`output/playwright/windows-launcher-corners-light.png`、`output/playwright/windows-launcher-corners-dark.png`。仅 CSS 和平台 class 变更，无窗口尺寸、AI、搜索、保存或复制逻辑变更。Windows 原生 WebView 合成仍需新安装包实机确认，本轮未发布。
