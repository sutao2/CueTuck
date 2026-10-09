# Windows 启动器样式修复

## 方案

截图显示系统方形边缘与内部圆角不一致，AI 草稿页在默认 620×420 下出现多余滚动，底栏拥挤。Tauri 本地依赖的 shadow 文档明确指出 Windows 无边框窗口开启阴影会添加 1px 白边。

1. 仅 Windows 启动器禁用原生 shadow，保留透明独立窗口及内部边界；其他窗口与平台保持既有配置。
2. 收紧草稿页最小高度和反馈间距，正文内部滚动、底栏留足间距；长原文展开仍允许中部滚动。
3. 验证创建、AI 失败保留原文、结果编辑、返回、保存与变量使用回归；浏览器检查 620×420、明暗主题和长正文。原生外框效果需 Windows 实机验证，不能用浏览器截图代替。

## 验收

- 完整前端门禁通过：桌面 678、网页 33、管理台 106 项测试及三端构建，日志 `output/verification/run-Na3usw/`。Mac 原生库测试 140 项通过、6 项既有忽略。
- Playwright 固定 AI 返回值、Windows 平台标识、620×420 验收：短结果正文区 clientHeight/scrollHeight 均 310px，无多余整体滚动；长结果编辑框内部滚动，展开原文时底栏保持 y=370..420。浅色和深色截图分别为 `output/playwright/launcher-style-short.png`、`output/playwright/launcher-style-long-dark.png`。
- Windows 原生 shadow 修复依据已安装 Tauri 的 API 文档；未运行 Windows 安装包，本轮浏览器与 Mac 验收不能证明 Windows 系统外框效果，需后续 Windows 实机确认。
- `cargo fmt --check` 遇到仓库既有多文件格式差异，未扩大修改；`git diff --check` 通过。
- 本次不改变 AI 请求、复制或保存语义，不发布新版本。
