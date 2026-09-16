# 发行前手工 QA

以下表格是 2026-08-23 的历史 smoke，不代表当前完整发行验收。2026-09-08 本机自动回归、恢复与调试构建结果见 [P4–P6 收尾记录](../plans/2026-09-08-local-release-readiness.md)。当前顶栏搜索已改为软件内搜索，独立启动器入口在底栏；签名、外部提供商和其他 OS 实机仍待验。

以下为历史稳定版 smoke 清单；未完成对应验收不得打稳定发行标签。明确授权的手动安装预览版按[当前预览发行计划](../plans/2026-09-13-github-release.md)限定平台、记录签名边界并标记 prerelease，不冒充稳定发行。不要把未跑过的平台勾成通过。

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 机器 | macOS darwin 27.0.0 |
| 构建 | 浏览器 `http://localhost:1420/` + `npm test`（26）+ `cargo test --locked`（21 通过，1 ignored） |
| 商店包 | 无 |
| 公开下载 | 不声称 |

## 清单

| 项 | 结果 | 备注 |
|---|---|---|
| 新建提示词后出现在本地列表 | 通过 | 浏览器新建「M4备份验证」，侧栏「本地提示词 1」 |
| 分类筛选不缩小侧栏「本地 N 条」 | 通过 | 点「网站开发」内容为空，侧栏仍为 1 |
| 启动器是独立窗口 / `/launcher.html` | 通过 | 顶栏搜索打开 `/launcher.html`，不是主窗口遮罩 |
| 启动器空查询无列表；Esc 清空 | 通过 | 空查询文案「空查询不展示列表」，无 listbox |
| 设置快捷键冲突时报错且不保存 | 跳过 | 需 `tauri dev` 登记全局快捷键 |
| JSON 导入先预览再写入 | 通过 | 「将导入 2 条提示词…确认前不会写入」，侧栏仍为 1 |
| 库文件备份后恢复只剩备份内容 | 通过 | Rust `restore_replaces_library` |
| 恢复无效文件不改库 | 通过 | Rust `failed_restore_leaves_library` |
| 浏览器点备份见「仅桌面窗口支持库文件备份」 | 通过 | 设置 → 数据与备份 |
| 广场离线说明与回本地 | 通过 | 「当前离线」+「前往本地」；启动器搜索不请求广场 |
| 未登录下载 / 收藏分离 | 通过 | 下载不弹登录，本地出现「自然光群像」；收藏原因含「收藏」，本地仍 1 条 |
| 发布未选源禁用；提交后本地可改 | 通过 | Vitest：submit 未选源 disabled；提交后仍能改正文 |
| 未验证 Windows / Linux | 跳过 | 开机启动与托盘已写出行为；本机未做 Windows / Linux 手工 smoke，不得宣传为已支持，不得勾通过 |

## 未做

- 远程 Playwright CI 执行（工作流已接入，本机执行与远端执行分开记录）
- 安装包签名与商店上架
- 非 macOS 安装包验证
- 本机 `tauri dev` 全局快捷键冲突（设置页单测已覆盖注册失败不写入）
- 系统红绿灯仅 `tauri dev` 可见；浏览器已验顶栏 inset 与 `⌃Space`


## Mac 预览签名身份

Mac 预览构建必须复用固定签名身份，禁止 `-` / ad-hoc 回退。`npm --prefix desktop run build:preview` 接受显式 `APPLE_SIGNING_IDENTITY` 及 Tauri 标准证书环境变量，或读取本机受限 `output/private/macos-signing/` 中的 identity.json、signing.p12 和 password。这些材料必须独立安全备份，构建不得自动重新生成证书；不得提交、上传为发行附件或打印密码。Windows 构建不要求 Apple 身份。

固定自签名仅供明确标记的预览包保持跨版本身份，不能替代 Developer ID、公证或 Gatekeeper 认证。发布前验证新旧二进制 DR 一致、隔离钥匙串跨版本读取与不同签名拒绝；迁移自旧 ad-hoc 后可能需要用户首次授权。当前证据见 [重复授权修复](../plans/2026-09-16-macos-signing-identity.md)。
