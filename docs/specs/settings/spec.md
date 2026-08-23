# 设置

| 字段 | 值 |
|---|---|
| 状态 | 已指定；M2 已实现本机子集；M8 已对齐原型十类 |
| 来源 | 原型设置弹窗信息架构；行为以本机为准，不得假装云能力已接通 |
| 关联 | [启动器](../launcher/spec.md) · [认证](../auth/spec.md) · [M8](../../plans/milestones/m8.md) |

## Purpose

用弹窗管理本机偏好。设置 MUST 是工作台弹窗，不是旧产品那套独立设置路由站。

M2 已落地且 MUST 保留：常规入口、启动器全局快捷键、JSON 导入预览与导出、库文件备份与恢复、浅色/深色主题。M8 只增加，不删除这些行为。

## Requirements

### Requirement: 入口

系统 MUST 从侧栏底部打开设置。

#### Scenario: 打开设置

- GIVEN 工作台已显示
- WHEN 用户点击「设置」
- THEN 出现设置弹窗
- AND 左侧为分类，右侧为当前页

### Requirement: M2 已交付页面

下列页面在 M2 已有真实行为，后续里程碑 MUST 继续提供，不得改成纯说明页：常规、快捷键、数据与备份、外观。

账号与广场、同步在 M2 可以显示「将在联网版提供」，MUST NOT 假装已接通。该占位 MUST 保留到对应行真正接通为止，不得用删页代替。

#### Scenario: 未实现页

- GIVEN 云同步尚未提供
- WHEN 用户打开「同步」
- THEN 页面说明尚未提供云同步
- AND 不调用后端

### Requirement: 导航十类

M8 起左侧 MUST 固定十类，顺序与原型一致：常规、账号与广场、快捷键、同步、AI 与模型、数据与备份、网络与代理、外观、隐私与安全、更新。

不得用合并、改名或删除来减少分类。尚未接通的分类 MUST 仍可打开，并展示原型中的行；云能力行 MUST 标明尚未提供，MUST NOT 假装已接通。

#### Scenario: 十类都在

- GIVEN 用户打开设置
- WHEN 查看左侧分类
- THEN 可见上述十类且均可选中

#### Scenario: 缺少的页不得消失

- GIVEN 自动更新安装尚未实现
- WHEN 用户打开「更新」
- THEN 页面仍显示当前版本、检查更新、自动下载、更新通道、发行说明各一行
- AND 「检查更新」不假装已经从商店或更新服务器取得结果

### Requirement: 常规

系统 MUST 提供并可持久化：开机启动、关闭后最小化到托盘、使用后自动关闭快捷窗口。未验证的操作系统 MUST NOT 声称该行已生效。

#### Scenario: 开机启动

- GIVEN 用户在常规页打开开机启动且当前平台已实现
- WHEN 系统保存成功
- THEN 下次登录系统后应用会启动
- AND 保存失败时开关回到原状并说明原因

### Requirement: 快捷键

系统 MUST 能记录并保存启动器全局快捷键。与系统冲突时 MUST 提示失败，不得静默无效。该行为 M2 已交付，MUST 保留。

M8 起快捷键页 MUST 另有：新建提示词、快速粘贴最近使用。默认组合与原型一致（macOS 上用 Mac 符号展示）。未实现的组合 MUST 仍显示该行，不得从页上删掉。

#### Scenario: 保存快捷键

- GIVEN 用户在快捷键页录制新组合
- WHEN 系统注册成功
- THEN 该组合能唤起启动器
- AND 旧组合不再唤起

#### Scenario: 新建与粘贴快捷键可见

- GIVEN 用户打开快捷键页
- WHEN 查看全局快捷键分组
- THEN 可见唤起快捷搜索、新建提示词、快速粘贴最近使用三行

### Requirement: 数据

系统 MUST 提供 JSON 导出、带预览的 JSON 导入、备份与恢复。导入 MUST 先预览再写入。失败 MUST 可取消且不写半份数据。该行为 M2/M4 已交付，MUST 保留。

M8 起数据与备份页 MUST 另有：打开库文件所在目录、导出完整 ZIP（提示词、合集、分类、封面与设置）、自动备份。ZIP 与自动备份是增加项，不得替换 JSON 或库文件备份。

#### Scenario: 导入预览

- GIVEN 一份含 2 条提示词的 JSON
- WHEN 用户选择导入
- THEN 先看到 2 条预览
- AND 确认前数据库条数不变

#### Scenario: 备份恢复

- GIVEN 库里有提示词 A，并已备份库文件
- WHEN 再写入提示词 B 后恢复该备份
- THEN 库里只剩 A
- AND 恢复无效文件时库仍是恢复前的内容

#### Scenario: 打开目录与 ZIP 行可见

- GIVEN 用户打开数据与备份页
- WHEN 查看本地数据分组
- THEN 可见 SQLite 路径与打开目录、导出完整备份、自动备份
- AND 仍可见原有 JSON 导入导出与库文件备份恢复

### Requirement: 外观

系统 MUST 支持浅色/深色，并持久化到 `settings` 表。启动器 MUST 读取同一主题键。该行为 M2 已交付，MUST 保留。

M8 起外观页 MUST 另有：跟随系统、界面语言（中文 / English）、提示词双语版本、内容密度。增加跟随系统不得删除浅色/深色选项。

#### Scenario: 切换主题

- GIVEN 用户在外观页选择深色
- WHEN 系统保存
- THEN 再次读取 theme 为 `dark`
- AND 启动器读取同一键

#### Scenario: 外观增加项可见

- GIVEN 用户打开外观页
- WHEN 查看主题与语言分组
- THEN 可见浅色、深色、跟随系统
- AND 可见界面语言、提示词双语版本、内容密度

### Requirement: 账号与广场

账号与广场页 MUST 展示：当前账号、作者主页、我的发布、下载时保留作者信息。

当前账号 MUST 接到已有邮箱密码登录/登出，不得改成未选定的 OAuth。作者主页与我的发布在对应产品面未交付前 MUST 标明尚未提供，MUST NOT 假装已编辑资料或已列出远端发布。下载时保留作者信息是本机开关，接通后 MUST 影响本机副本展示，未接通时仍保留该行。

#### Scenario: 当前账号接已有登录

- GIVEN 用户已用邮箱登录
- WHEN 打开账号与广场页
- THEN 「当前账号」显示该邮箱或已登录态，并可登出
- AND 不出现 QQ / LinuxDo / Google 绑定入口

### Requirement: 同步

同步页 MUST 展示原型四行：自动同步收藏与发布草稿、仅在 Wi-Fi 下同步图片、冲突处理、立即同步。云同步引擎未立项前，这些行 MUST 标明尚未提供，MUST NOT 调用后端或写入假的「已同步」。

#### Scenario: 同步行可见且不假装

- GIVEN 云同步尚未提供
- WHEN 用户打开同步页
- THEN 四行都在
- AND 立即同步不向服务器发请求

### Requirement: AI 与模型

AI 与模型页 MUST 展示：默认目标模型、已启用模型库、显示模型标签、变量智能建议、自定义模型列表。这些是本机目录、标签与建议，MUST NOT 把提示词正文上传到模型供应商。变量智能建议关闭时 MUST 不提供建议。

#### Scenario: 模型页可见且不外传正文

- GIVEN 用户打开 AI 与模型页
- WHEN 查看使用偏好
- THEN 五行星都在
- AND 打开变量智能建议不会把提示词正文发到本机以外

### Requirement: 网络与代理

网络与代理页 MUST 展示：允许访问提示词广场、代理、同步状态。「允许访问提示词广场」接通后 MUST 成为本机开关：关闭时工作台 MUST 不请求广场，启动器仍 MUST 只搜本地。代理与同步状态在未实现手动配置或云同步前 MUST 标明现状（例如跟随系统 / 尚未提供），MUST NOT 假装正在同步。

#### Scenario: 关闭广场访问

- GIVEN 用户关闭「允许访问提示词广场」且该开关已接通
- WHEN 在工作台打开广场
- THEN 不请求广场 API
- AND 启动器搜索仍只走本地库

### Requirement: 隐私与安全

隐私与安全页 MUST 展示：本地提示词默认不上传、匿名下载统计、清除使用历史、系统钥匙串。默认不上传 MUST 与宪法一致：未点发布不得把本地正文送出。匿名下载统计未接通前 MUST 标明尚未提供，不得静默上报。清除使用历史接通后 MUST 只删最近使用记录，不得删提示词正文。钥匙串行 MUST 反映 Refresh 是否在系统密钥库，不得把 Refresh 改存 Web Storage。

#### Scenario: 清除使用历史不删正文

- GIVEN 库里有提示词 A，且存在使用记录
- WHEN 用户确认清除使用历史且该动作已接通
- THEN 使用记录被清除
- AND 提示词 A 仍在

### Requirement: 更新

更新页 MUST 展示：当前版本、检查更新、自动下载更新、更新通道、发行说明。当前版本 MUST 为真实应用版本。其余行在无更新通道前 MUST 标明尚未提供，MUST NOT 假装已检查商店或已排队安装。

#### Scenario: 版本真实、检查不假装

- GIVEN 用户打开更新页
- WHEN 查看应用更新
- THEN 当前版本与本机构建一致
- AND 检查更新不声称已经连上更新服务器

## 测试映射

| 场景 | 测试 |
|---|---|
| 打开设置 | `WorkbenchShell.spec.js` opens settings from the sidebar |
| 未实现页 | 同上，打开「同步」见 `settings-unavailable` |
| 十类都在 | `WorkbenchShell.spec.js` lists ten settings categories |
| 缺少的页不得消失 | `WorkbenchShell.spec.js` keeps the updates page without claiming a store check |
| 开机启动 | `desktopPrefs.test.js` persists launch at login on macos；`WorkbenchShell.spec.js` saves launch at login on macos / does not claim launch at login on windows |
| 保存快捷键 | `shortcut.test.js` does not persist when register throws |
| 新建与粘贴快捷键可见 | `WorkbenchShell.spec.js` shows new and paste shortcut rows |
| 导入预览 | `desktop/src-tauri` `import_preview_does_not_write`；`library.test.js` previews import without writing |
| 备份恢复 | `desktop/src-tauri` `restore_replaces_library`、`failed_restore_leaves_library`；`library.test.js` rejects sqlite file backup in the browser memory library |
| 打开目录与 ZIP 行可见 | `WorkbenchShell.spec.js` shows open directory and zip rows with existing backup actions；`library.test.js` exports a zip payload without dropping memory prompts；`desktop/src-tauri` `export_zip_does_not_remove_sqlite`、`auto_backup_leaves_existing_backup` |
| 切换主题 | `desktop/src-tauri` `theme_persists_as_dark` |
| 外观增加项可见 | `WorkbenchShell.spec.js` shows appearance extras including follow-system theme |
| 当前账号接已有登录 | `WorkbenchShell.spec.js` shows the current account from the existing login |
| 同步行可见且不假装 | `WorkbenchShell.spec.js` shows sync rows without requesting the backend |
| 模型页可见且不外传正文 | `WorkbenchShell.spec.js` shows model rows without sending prompt bodies |
| 关闭广场访问 | `WorkbenchShell.spec.js` does not request square when access is off |
| 清除使用历史不删正文 | `WorkbenchShell.spec.js` clears use history without deleting prompt content；`library.test.js` clears use counts without deleting prompt content；`desktop/src-tauri` `clear_use_history_keeps_prompt_content` |
| 版本真实、检查不假装 | `WorkbenchShell.spec.js` keeps the updates page without claiming a store check |
