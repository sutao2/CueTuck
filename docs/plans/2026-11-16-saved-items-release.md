# 常用资料与 beta.38 发布

1. 工作台、独立启动器、全局搜索和官网统一使用「常用资料」；仅改展示名称，保留现有存储和凭据标识。
2. 列表复用工作台标题、搜索、筛选和状态组件，行内操作沿用既有约定，核对宽窄窗口、浅深色与分类数量。
3. 运行前端与原生回归，从同一提交构建 Windows x64 和 macOS Apple Silicon 候选安装包。在隔离旧数据库上验证新增、启动器归类、搜索与跨窗刷新。生成候选附件、校验和及双平台更新清单。

4. 使用用户指定的[蓝色卡片收纳图标](https://chatgpt.com/s/m_6afa63b7d2d08191bf1abdc43d1fad97)作为位图母版，保留原画，按桌面图标规范处理外缘。验证启动器、官网资源、Dock 和 Windows 安装包内的实际图标。

5. Windows 安装验收出现锁屏停在「重试」的情形。锁状态临时不可读时先保持工作台隐藏并自动重试一次；持续失败时保留手动重试，覆盖两种结果后复验安装包。

6. 修复应用更新检查。保留 GitHub Releases 作为发行来源，直连失败后从官网固定地址重取同一公开列表。测试超时、限流、无效响应、两个来源都失败，以及稳定和预览通道；安装仍按所选版本读取和验证签名包。

7. 工作台和启动器的文本、模板保留换行、缩进和空行；编辑区适配长内容。已有资料可原位改类型，保留标识与分类。跨密码边界须明确输入新内容；验证凭据失败、过期编辑和取消都保留原记录。

## 候选验收

`0.1.0-beta.38` 的 macOS Apple Silicon 与 Windows x64 安装包来自 `bd58c78`。macOS 从旧记录原位转换模板后逐字保留 CRLF，独立启动器保存多行样例后与输入一致。Windows 安装包的文本和模板通过了类型转换与剪贴板逐字比对。两个系统的设置页均从公开发行源收到「当前已是此通道最新版本」。更新签名、macOS 隔离安装及校验和均已验证。[全量回归](https://github.com/sutao2/CueTuck/actions/runs/43792897079) · [Windows 安装包复验](https://github.com/sutao2/CueTuck/actions/runs/43794219522)

## 2026-11-16 发布验收

- [beta.38 预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.38)的标签指向已验收的 `bd58c78`。8 个公开附件均已匿名下载并与候选文件逐字节比对，两个平台的更新包签名通过校验。macOS 更新器通过公开 HTTPS 清单下载并完成隔离安装。
- [Windows beta.37 安装包复测](https://github.com/sutao2/CueTuck/actions/runs/43795072252)使用与公开包相同的 SHA-256，从设置点击「检查更新」后出现「下载更新」。GitHub 和官网两份公开列表均能选中 beta.38，同版无需更新，稳定通道排除该预览版。
- [官网下载页](https://prompt.likh.cn/#download)的 macOS 与 Windows 按钮已核对，[官网备用更新源](https://prompt.likh.cn/data/desktop-releases.json)已返回 beta.38。官网当前目录为 `/opt/cuetuck-website-preview/releases/beta38-bd58c78`，上一目录 `beta37-bfae7e7` 保留供回退。官网容器及原有 API 健康检查通过。
