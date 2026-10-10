# 官网

## Purpose

以独立启动器为主线介绍 CueTuck，提供已发布客户端的下载和公开内容浏览入口。根域名与数据能力边界沿用[官网入口记录](../../plans/2026-09-15-launcher-website.md)和[公开数据合同](../../plans/2026-09-15-website-live-data.md)。

## Requirements

### Requirement: 清晰的产品表达

官网 MUST 使用中性白灰、清晰的标题与正文层级、圆润主操作及充分留白；CueTuck 标识保留自身品牌。首屏以桌面启动器价值和真实产品截图为重点，下载为主操作、公开广场为次操作。MUST NOT 使用虚构用户量、客户标识或证言。

- Given 初次访问 When 阅读首页 Then 明确全局唤起、搜索、填写复制的用途，不将其描述为自动粘贴或聊天模型服务。
- Given 操作预览 When 切换搜索/填写 Then 对应真实截图与可访问选中状态同步，键盘可完成操作。

### Requirement: 一致与可达

首页和下载页 MUST 共享导航、字号、按钮和留白体系，提供 macOS Apple Silicon 与 Windows x64 的准确下载入口。共用样式 MUST 保留既有广场和 Skills 的数据与交互。

- Given 390px、960px 或 1440px 视口 When 浏览各页面 Then 无页面横向溢出，导航和主要操作可达，焦点可辨识，图片保持比例。
- Given 下载页 When 选择平台 Then 下载当前发行对应资产，并能查看预览版系统签名限制和校验文件。

本轮实现及部署证据见[视觉优化计划](../../plans/2026-10-10-website-polish.md)。

广场的卡片层级与有界首屏加载遵循[广场规格](../square/spec.md)，官网保留真实分页、分类与本页暂存的现有边界。
