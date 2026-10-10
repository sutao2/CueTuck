# Windows 启动器边界与阴影

## 方案

用户截图中启动器与背后的浅色页面难以区分，参考图采用浅灰面板、清晰细边框和窗口投影。本切片仅调整 Windows 独立启动器外观。

1. Windows 原生窗口改为不透明背景并启用系统 shadow，由系统绘制外部阴影及支持时的圆角；CSS 继续铺满矩形客户区，避免此前透明圆角黑块。接受 Tauri 在 Windows 无边框阴影窗口附加的系统细边缘，Windows 10 不承诺圆角。
2. Windows 面板使用随主题变化的轻微灰底，增加覆盖整个面板的细边框，确保列表、草稿、底栏都不会遮住外轮廓；不改变窗口尺寸和业务操作。
3. 验证启动器回归、前端构建及文档门禁；浏览器检查 Windows 浅深色、收起/展开/填写和 Mac 样式隔离。系统阴影与原生圆角必须单列 Windows 实机验收边界。

## 场景

- Given Windows 浅色主题且背后是白色页面 When 启动器展开 Then 浅灰面板与完整灰色细边框能够区分窗口范围，系统阴影提供外部层次。
- Given Windows 深色主题 When 搜索、填写或编辑草稿 Then 外框持续可见，底栏不遮住轮廓，内容无横向溢出。
- Given Windows 空查询 When 窗口收起为 64px Then 搜索框和完整外框均可见，输入后仍按原偏好展开。
- Given Windows 11/10 When 原生窗口显示 Then 系统按平台能力提供圆角/直角，客户区保持实色而不依赖 CSS 透明角。
- Given Mac 启动器 When 唤起、搜索或填写 Then 保持已有透明画布、18px 圆角与原生阴影。

## 验收

- 52 项启动器测试通过（搜索、排序、变量填写、快捷创建/AI 与窗口尺寸），前端生产构建通过。
- 本机 `cargo check --offline --lib` 通过，保留既有未使用项警告；使用 Command Line Tools 避开当前 Xcode 许可未确认的问题。此检查为 Mac 目标，不代表 Windows 编译或实机验收。
- Playwright 隔离页面同时模拟 Windows platform/userAgent：620×420 浅深色列表、变量填写与创建草稿，620×64 收起状态均已截图检查。外框为完整 1px、无横向溢出，底栏止于 420px；边框不拦截点击。深色保留原面板底色以维持选中行对比。Mac 分支仍为 18px 圆角、透明画布且无新增边框。
- 截图位于 `output/playwright/windows-outline-{light,dark,fill-dark,draft-dark,collapsed}.png`。浏览器截图只覆盖客户区，不能显示或证明 Windows 系统阴影/圆角；原生能力依据已安装 Tauri 2.11.5 的 shadow API 文档，Windows 10/11 最终效果待安装包实机复验。
- `scripts/docs-check` 与 `git diff --check` 通过。后续发行见 [beta.25 发布记录](2026-10-10-beta25-release.md)。
