# 启动器焦点修复与 beta.39 发布

## 计划（设计已完成）

1. 从已提交的搜索框焦点修复构建 beta.39，版本、官网入口和更新清单保持一致。
2. 在 macOS DMG 与 Windows 安装包中验收空查询搜索栏、多行资料录入、文本与模板复制、应用锁和检查更新。保留安装后真实启动器截图，核对浅深色焦点外观。
3. 双平台包来自同一提交，验收通过后按预览发行发布签名附件。核对匿名下载字节、旧版发现新版、同版不提示及稳定通道筛选。
4. 按宝塔既有站点部署官网与固定版本列表入口，保留 beta.38 目录回退；核对公开下载按钮、版本列表和 API 健康。

## 验收场景

- Given 新安装包的启动器自动聚焦，When 空查询或输入，Then 搜索框无内嵌蓝色矩形，快捷操作仍能用键盘选择。
- Given 多行文本包含空行、缩进和尾部空格，When 添加资料、切换文本或模板、保存和复制，Then 正文保持一致。
- Given beta.38 已安装并使用预览通道，When 检查更新，Then 能发现 beta.39；beta.39 已安装时无重复升级，稳定通道不选预览版。

## 2026-11-16 发布验收

- [beta.39 预览发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.39)标签指向同源提交 `dfd1b78`，8 个公开附件全部匿名下载并与已验收文件逐字节比对。两个平台的签名包已校验，macOS 更新器从公网 HTTPS 清单下载并完成隔离安装。
- [Windows 构建与安装验收](https://github.com/sutao2/CueTuck/actions/runs/43797612133)通过原生测试、凭据库读写、便携 ZIP、NSIS 安装运行卸载、资料分类、类型转换、多行复制、应用锁和设置页更新检查。Windows 与 macOS 安装后真实截图中，浅深色启动器均无蓝色矩形，macOS 录入模板仍逐字保留空白。截图保存在 `output/verification/beta39/launcher-{windows,macos}-{light,dark}.png`。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/43797612151)通过，本机前端 760 项通过。[原版 Windows beta.38 实测](https://github.com/sutao2/CueTuck/actions/runs/43798757977)在设置点击「检查更新」后出现「下载更新」；公开 GitHub 与官网列表均验证旧版发现新版、同版不提示、稳定通道排除预览版。
- [官网下载页](https://prompt.likh.cn/#download)两个安装包入口均已实测，[备用更新源](https://prompt.likh.cn/data/desktop-releases.json)已提供 beta.39。官网目录为 `/opt/cuetuck-website-preview/releases/beta39-dfd1b78`，上一目录 `beta38-bd58c78` 可供回退。官网服务仍由 `cuetuck-website-preview` 管理，监听 `127.0.0.1:15175`，容器及 API 健康通过，临时上传文件已清理。
- 发行说明保留 macOS 公证、Windows 发布者签名与 Windows Hello 已录入设备真机待验的边界。
