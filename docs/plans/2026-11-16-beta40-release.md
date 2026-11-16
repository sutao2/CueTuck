# Windows 图标升级修复与 beta.40 发布

## 设计（已完成）

1. beta.40 集成独立 ICO 快捷方式修复，Windows 与 macOS 从同一提交构建。
2. Windows 从公开双环版 beta.37 原位升级，核对安装包实际执行修复后桌面、开始菜单与已有任务栏快捷方式的图标路径及内容。验收直接检查成品安装包，并保留窗口图标与快捷方式图像。
3. 验收 macOS DMG 的固定签名、系统图标、启动器浅深色、多行资料、应用锁和更新检查；Windows 同时验收 NSIS、便携 ZIP 和相关既有功能。
4. 验收通过后发布签名附件，核对旧版发现新版、同版不重复、稳定通道筛选和公网更新安装，切换官网下载入口与备用更新源，保留 beta.39 回退目录。

## 验收场景

- Given 旧双环版及已固定的快捷方式，When 安装 beta.40，Then 本应用快捷方式指向当前可执行文件，并使用内容摘要命名的蓝色卡片 ICO，窗口图标与快捷方式图标一致。
- Given macOS 安装包，When 启动、搜索、保存多行资料与锁定解锁，Then 功能正常，系统图标为现行蓝色卡片。
- Given 已安装 beta.39，When 使用预览通道检查更新，Then 可发现 beta.40；beta.40 当前版本不重复更新。

## 发布验收（已完成）

- 发布源为 `ef4250010c7bba8fa81d95284f61c97082daca7b`。[完整回归](https://github.com/sutao2/CueTuck/actions/runs/43812137697)和 [Windows 构建与安装验收](https://github.com/sutao2/CueTuck/actions/runs/43812137748)均通过；NSIS、便携 ZIP、旧资料多行编辑与类型转换、启动器浅深色及口令锁已验收。
- [成品安装包图标升级验收](https://github.com/sutao2/CueTuck/actions/runs/43813217770)使用公开 beta.37 在同一目录覆盖安装 beta.40，保留任务栏固定项文件夹中的旧版快捷方式样本。安装器自行修复后，三个快捷方式均保留正确目标，独立 ICO 与仓库摘要一致，重新读取并导出的桌面、开始菜单和任务栏快捷方式图像均为蓝色卡片；本次验收未额外执行修复脚本。
- macOS DMG 已验证可挂载、固定证书签名与稳定应用标识；隔离安装后系统图标、启动器、保留 CRLF/空行/制表符的编辑与类型转换、错误及正确口令、冷启动解锁和设置页检查更新均通过。
- [Windows 已发布 beta.39 更新检查](https://github.com/sutao2/CueTuck/actions/runs/43813549082)实际点击设置页按钮后显示「下载更新」。主源与备用源同时验证 beta.39 发现 beta.40、beta.40 不重复更新、稳定通道排除预览版。macOS 真实更新器已从公开 HTTPS 源下载、校验签名并安装到临时应用；修改附件内容会被签名校验拒绝。
- [beta.40 发行页](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.40)保留预发布标记，共八个公开附件。匿名下载的每个文件与验收成品逐字节一致，`SHA256SUMS` 与平台更新签名均已核对。成品和图像证据分别保存在 `output/releases/v0.1.0-beta.40` 与 `output/verification/beta40`。
- [官网下载页](https://prompt.likh.cn/#download)的 Windows 与 macOS 入口、图标及 `/data/desktop-releases.json` 已上线。服务器当前目录为 `/opt/cuetuck-website-preview/releases/beta40-ef42500`，保留 `beta39-dfd1b78` 回退目录；生产容器健康，继续监听 `127.0.0.1:15175`。临时上传和验收浏览器已清理，本次未保留本地开发服务。

## 仍适用的边界

少数已有任务栏固定项仍可能缓存图像，可取消固定后从已修复的桌面快捷方式重新固定。发布说明已保留 macOS 尚未 Apple 公证、Windows 尚未 Authenticode 签名、Windows Hello 已录入设备真机验收待完成的说明。
