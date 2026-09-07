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

### Requirement: 保存反馈与安全退出

即时开关和选择项 MUST 在保存成功后更新生效值；失败恢复原值并在固定反馈区说明失败。进行中禁止重复提交与关闭。模型、快捷键与作者资料为显式保存，MUST 标明保存方式，成功或失败均有可见反馈。含未保存表单或导入文本时关闭设置 MUST 先询问，取消保留输入，不自动提交。

#### Scenario: 保存失败与未保存退出

- GIVEN 设置已有持久化值，用户修改开关或表单
- WHEN 写入失败或未保存就关闭
- THEN 失败的即时选项回到原值；未保存退出先确认，选择继续编辑后输入仍在
- AND 不将失败或部分保存写成全部保存成功

#### Scenario: 危险操作确认

- GIVEN 用户打开数据或隐私页
- WHEN 点击恢复库文件或清除历史
- THEN 显示操作范围的二次确认，取消不写数据
- AND 只有确认后执行；恢复说明会替换当前库，清除历史说明不删除提示词

#### Scenario: 密度真实生效

- GIVEN 已打开工作台
- WHEN 内容密度在舒适与紧凑之间切换
- THEN 主窗口卡片、列表与分类行的间距改变，重新打开仍沿用已保存密度
- AND 不改变独立启动器布局

### Requirement: 入口

系统 MUST 从侧栏底部打开设置。

#### Scenario: 打开设置

- GIVEN 工作台已显示
- WHEN 用户点击「设置」
- THEN 出现设置弹窗
- AND 左侧为分类，右侧为当前页

#### Scenario: 切页与长文字不改变弹窗尺寸

- GIVEN 设置弹窗已打开
- WHEN 切换任一设置页、显示较长提示或调整文本框高度
- THEN 弹窗保持 760 × 600 CSS 像素，视口较小时收缩到四周至少 24 像素边距
- AND 标题与关闭按钮保持可见，导航和内容超出时分别内部滚动，不随正文撑大窗口

### Requirement: 统一设置布局

设置名称 MUST 位于左侧导航顶部，右侧只显示当前页标题；关闭按钮保持可见。普通选项左说明右控件；多字段资料、账单与多行编辑 MUST 纵向分组，不挤在说明旁边。文本控件、选择框和开关有一致的尺寸、禁用与键盘焦点状态。

账号资料与兑换输入 MUST 有持续可见的字段标签，不仅依赖占位文字。十个页面的标题说明层级一致，长发布列表与多行发行说明可换行显示。

#### Scenario: 简单选项与复杂表单

- GIVEN 设置弹窗打开
- WHEN 切换常规、账号与广场、模型或数据页
- THEN 导航位置与弹窗尺寸不变，简单选项对齐，多字段内容独占可用宽度
- AND 所有原有设置入口、保存与错误提示保留，短视口只滚动内容，不裁掉关闭入口

### Requirement: M2 已交付页面

下列页面在 M2 已有真实行为，后续里程碑 MUST 继续提供，不得改成纯说明页：常规、快捷键、数据与备份、外观。

账号与广场、同步在 M2 可以显示「将在联网版提供」，MUST NOT 假装已接通。该占位 MUST 保留到对应行真正接通为止，不得用删页代替。

#### Scenario: 未实现页

- GIVEN 本机设置已打开
- WHEN 用户打开「网络与代理」
- THEN 代理行可填写地址且空则跟随系统
- AND 启动器与 MCP 仍只读本机 SQLite

### Requirement: 导航十类

M8 起左侧 MUST 固定十类，顺序与原型一致：常规、账号与广场、快捷键、同步、AI 与模型、数据与备份、网络与代理、外观、隐私与安全、更新。

不得用合并、改名或删除来减少分类。尚未接通的分类 MUST 仍可打开，并展示原型中的行；云能力行 MUST 标明尚未提供，MUST NOT 假装已接通。

#### Scenario: 十类都在

- GIVEN 用户打开设置
- WHEN 查看左侧分类
- THEN 可见上述十类且均可选中

#### Scenario: 缺少的页不得消失

- GIVEN 用户打开「更新」
- WHEN 查看页面
- THEN 页面仍显示当前版本、检查更新、自动下载、更新通道、发行说明各一行
- AND 「检查更新」不声称已经从商店取得结果

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

- GIVEN 用户聚焦任一快捷键录入框
- WHEN 按下修饰键与字母、数字、空格等主键的组合，或单独功能键
- THEN 自动录入标准组合，macOS 显示对应符号，不要求手打组合字符串
- AND 录制期间阻止本应用全局快捷键动作；单独修饰键、输入法组字和普通裸字母不覆盖原组合
- AND Tab/Shift+Tab 正常移动焦点，Esc 恢复本次聚焦前的组合并结束录制，不关闭设置；失焦与卸载解除保护
- AND 三项重复组合在注册前拒绝，系统保留键被系统拦截时不得声称已录入或已生效

#### Scenario: 注册快捷键

- GIVEN 用户在快捷键页录制新组合
- WHEN 系统注册成功
- THEN 该组合能唤起启动器
- AND 旧组合不再唤起
- AND 启动器注册与持久化完成后立即同步工作台标签（见[工作台规格](../workbench/spec.md)），不等待其他两项偏好写入；失败不把未保存草稿显示为生效值

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

### Requirement: 一致性备份与定期执行

备份和 ZIP MUST 使用 SQLite 一致性快照，包含已提交但尚在 WAL 的数据。备份失败 MUST 保留已有目标。恢复 MUST 先在副本上完成兼容迁移与完整性/结构校验，再原子写入当前库；失败 MUST 保留当前库，MUST NOT 修改输入备份。恢复路径不得为当前库。恢复成功 MUST 刷新界面和偏好，并提示快捷键、代理及系统级偏好重启应用后生效。ZIP 的 JSON 与 SQLite MUST 来自同一时刻快照；封面目前仅保存引用，不得声称包含外部图片文件。

自动备份 MUST 在原生应用运行时每分钟检查；开启后首次立即执行，此后距离最近成功至少 24 小时执行一次。文件存于应用数据目录的 backups 子目录，使用唯一名称保留旧备份。失败 MUST 可见且允许后续重试；关闭后 MUST 停止后续备份。浏览器 MUST 明确不支持并拒绝开启。

#### Scenario: WAL 与无效恢复

- GIVEN 当前库有 WAL 中的新提交，以及一份损坏或结构非法的恢复文件
- WHEN 先备份再尝试恢复无效文件
- THEN 备份包含新提交，无效恢复不改变当前库和输入文件
- AND 有写锁造成恢复失败时，当前库仍保持完整

#### Scenario: 自动备份到期

- GIVEN 原生应用运行且已开启自动备份
- WHEN 未到 24 小时、到期或随后关闭
- THEN 分别不重复备份、生成新的完整备份、停止后续执行
- AND 首次备份失败时显示错误，不把失败写成成功

#### Scenario: JSON 完整往返与原子导入

- GIVEN 导出库含自定义分类、合集封面和带模型的成员提示词
- WHEN 预览后确认导入该 JSON
- THEN 以新 id 创建副本，保留分类与成员关联、模型和封面，不覆盖已有条目
- AND 兼容仅含 title/content 的旧 JSON；任何非法标题、结构或关联导致整批失败，不留下半份数据
- AND 修改预览后的输入必须重新预览，成功后不可重复点击确认导入

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

#### Scenario: 界面语言切换壳层文案

- GIVEN 当前为中文
- WHEN 用户把界面语言改为 English 或点侧栏 EN
- THEN 工作台壳层与设置导航改为英文
- AND 再选中文后恢复

### Requirement: 账号与广场

账号与广场页 MUST 展示：当前账号、作者主页、我的发布、下载时保留作者信息。

当前账号 MUST 接到已有邮箱密码登录/登出。Google / GitHub 由登录弹窗按已配置提供商列出，见 [ADR 0013](../../architecture/decisions/0013-oauth-google-github.md)。MUST NOT 出现 QQ / LinuxDo 绑定入口。作者主页 MUST 能保存显示名与简介。我的发布 MUST 列出当前账号投稿。下载时保留作者信息接通后 MUST 影响本机副本展示。

#### Scenario: 当前账号接已有登录

- GIVEN 用户已用邮箱登录
- WHEN 打开账号与广场页
- THEN 「当前账号」显示该邮箱或已登录态，并可登出
- AND 不出现 QQ / LinuxDo 绑定入口

#### Scenario: 下载保留作者

- GIVEN 用户打开「下载时保留作者信息」
- WHEN 下载一条带作者的广场提示词
- THEN 本地副本展示该作者
- AND 关闭开关后新下载不展示作者

#### Scenario: 看到我的发布

- GIVEN 用户已登录且有一条 pending 发布
- WHEN 打开账号与广场页
- THEN 可见该条标识与状态

#### Scenario: 保存作者资料

- GIVEN 用户已登录
- WHEN 填写显示名并保存
- THEN 再次打开仍是该显示名
- AND 未登录不得写入

### Requirement: 同步

同步页 MUST 展示原型四行。个人库同步见 [同步规格](../sync/spec.md)。立即同步在已登录时 MUST 推拉账号库；未登录时 MUST 打开已有登录且 MUST NOT 出现已同步。冲突处理 MUST 提供较新者胜与保留本地，默认 MUST 为较新者胜，MUST NOT 写成尚未提供。仅 Wi-Fi 同步图片接通后 MUST 为本机开关：打开且网络不是 Wi-Fi（含无法判定）时 MUST 跳过合集封面，仍同步标题与提示词正文；MUST NOT 用空封面覆盖远端已有封面。该行 MUST NOT 标明尚未提供，MUST NOT 把原因写成没有云同步。自动同步收藏与发布草稿接通后 MUST 为本机开关：打开且已登录时，收藏或发布失败 MUST 写入本机队列且 MUST NOT 写成本地副本，MUST NOT 假装已经到达服务器；立即同步 MUST 冲刷本账号队列。开关关闭时失败 MUST 不入队。该行 MUST NOT 标明尚未提供，MUST NOT 把原因写成没有云同步。

#### Scenario: 同步行可见且不假装

- GIVEN 用户未登录
- WHEN 用户打开同步页并点立即同步
- THEN 四行都在
- AND 打开登录
- AND 不出现已同步
- AND 不调用同步接口

#### Scenario: 冲突处理标明较新者胜

- GIVEN 立即同步已接通且默认较新 `updated_at`
- WHEN 用户打开同步页
- THEN 冲突处理行标明较新者胜
- AND 该行不标明尚未提供
- AND 默认选项为较新者胜

#### Scenario: 冲突处理可选保留本地

- GIVEN 用户打开同步页
- WHEN 选择保留本地并立即同步，且远端同一 id 的 `updated_at` 更晚
- THEN 该条本机正文不被覆盖
- AND 设置再次打开仍是保留本地

#### Scenario: 仅 Wi-Fi 同步图片可开关

- GIVEN 用户打开同步页
- WHEN 打开仅 Wi-Fi 下同步图片
- THEN 该行不标明尚未提供
- AND 不出现没有云同步
- AND 再次打开仍是打开状态

#### Scenario: 自动同步收藏可开关

- GIVEN 用户打开同步页
- WHEN 打开自动同步收藏与发布草稿
- THEN 该行不标明尚未提供
- AND 再次打开仍是打开状态

### Requirement: AI 与模型

AI 与模型页 MUST 展示：默认目标模型、已启用模型库、显示模型标签、变量智能建议、自定义模型列表。这些是本机目录、标签与建议，MUST NOT 把提示词正文上传到模型供应商。变量智能建议关闭时 MUST 不提供建议。

#### Scenario: 模型页可见且不外传正文

- GIVEN 用户打开 AI 与模型页
- WHEN 查看使用偏好
- THEN 五行星都在
- AND 打开变量智能建议不会把提示词正文发到本机以外

#### Scenario: 显示模型标签接到卡片

- GIVEN 一条本地提示词带模型名且开关打开
- WHEN 用户看内容区卡片
- THEN 卡片展示该模型标签
- AND 关闭开关后标签消失

#### Scenario: 默认模型进入新建编辑器

- GIVEN 已启用模型库含 Flux 且默认目标模型为 Flux
- WHEN 用户新建提示词
- THEN 编辑器模型下拉含目录中的名称
- AND 默认选中 Flux

### Requirement: 网络与代理

网络与代理页 MUST 展示：允许访问提示词广场、代理、同步状态。「允许访问提示词广场」接通后 MUST 成为本机开关：关闭时工作台 MUST 不请求广场，启动器仍 MUST 只搜本地。代理接通后 MUST 可填写 http 或 https 地址：空 MUST 跟随系统；填写后本机 Tauri 请求 MUST 走该代理。非法地址 MUST NOT 保存。浏览器预览 MUST NOT 声称走该代理。MUST NOT 把 SOCKS 写成已支持。同步状态 MUST 标明个人库可立即同步，MUST NOT 写成没有云同步或尚未提供，MUST NOT 假装正在同步或已同步。

#### Scenario: 关闭广场访问

- GIVEN 用户关闭「允许访问提示词广场」且该开关已接通
- WHEN 在工作台打开广场
- THEN 不请求广场 API
- AND 启动器搜索仍只走本地库

#### Scenario: 空代理跟随系统

- GIVEN 代理地址为空
- WHEN 用户打开网络与代理
- THEN 该行标明跟随系统
- AND 不标明尚未提供

#### Scenario: 填写后本机走代理

- GIVEN 用户输入合法 http 代理地址
- WHEN 保存
- THEN 本机设置记下该地址
- AND 说明浏览器预览不走该代理

#### Scenario: 非法代理不保存

- GIVEN 用户输入不是 http 或 https 的地址
- WHEN 保存
- THEN 本机设置仍为空
- AND 说明地址无效

#### Scenario: 同步状态不写没有云同步

- GIVEN 立即同步已接通且无后台自动同步
- WHEN 用户打开网络与代理
- THEN 同步状态标明手动立即同步
- AND 不出现没有云同步
- AND 不出现已同步或正在同步

### Requirement: 隐私与安全

隐私与安全页 MUST 展示：本地提示词默认不上传、匿名下载统计、清除使用历史、系统钥匙串。默认不上传 MUST 与宪法一致：未点发布不得把本地正文送出。匿名下载统计接通后 MUST 为本机开关，默认 MUST 关闭。打开且本地下载成功后 MUST `POST /v1/square/items/{id}/downloads`，MUST NOT 带 Authorization，MUST NOT 发送账号、正文或标题。关闭时 MUST NOT 请求该接口。统计失败 MUST NOT 阻断下载。GET 正文 MUST NOT 当成统计。该行 MUST NOT 标明尚未提供，MUST NOT 静默上报。清除使用历史接通后 MUST 只删最近使用记录，不得删提示词正文。钥匙串行 MUST 反映 Refresh 是否在系统密钥库：Tauri 下标明本机钥匙串；浏览器预览 MUST NOT 写成本机钥匙串，MUST 说明 Refresh 不进 Web Storage。不得把 Refresh 改存 Web Storage。

#### Scenario: 匿名下载统计默关且可打开

- GIVEN 用户打开隐私与安全
- WHEN 查看匿名下载统计
- THEN 该行是关闭的开关
- AND 不标明尚未提供

#### Scenario: 打开后成功下载只上报条目 id

- GIVEN 匿名下载统计已打开
- WHEN 用户成功下载一条广场提示词
- THEN 客户端对该 id 发出匿名 POST
- AND 请求不含 Authorization、账号或正文

#### Scenario: 关闭时不请求统计

- GIVEN 匿名下载统计关闭
- WHEN 用户成功下载一条广场提示词
- THEN 不请求下载统计接口

#### Scenario: 清除使用历史不删正文

- GIVEN 库里有提示词 A，且存在使用记录
- WHEN 用户确认清除使用历史且该动作已接通
- THEN 使用记录被清除
- AND 提示词 A 仍在

#### Scenario: 浏览器预览不写本机钥匙串

- GIVEN 工作台在无 Tauri 的浏览器预览
- WHEN 用户打开隐私与安全或登录弹窗
- THEN 钥匙串行与登录脚注不标明本机钥匙串或只写入系统钥匙串
- AND 说明 Refresh 不进 Web Storage

#### Scenario: Tauri 标明本机钥匙串

- GIVEN 工作台在 Tauri
- WHEN 用户打开隐私与安全
- THEN 钥匙串行标明本机钥匙串
- AND 说明 Refresh 不进 Web Storage

### Requirement: 更新

更新页 MUST 展示：当前版本、检查更新、自动下载更新、更新通道、发行说明。当前版本 MUST 为真实应用版本。检查更新 MUST 请求 GitHub Releases；稳定通道只用正式发行，预览通道只用预发行。无对应发行物或已是最新时 MUST 说明没有可用更新。读取失败时 MUST 说明检查失败，不得写成没有可用更新。自动下载是本机开关；打开且当前通道有包时 MUST 通过 Tauri updater 排队安装，MUST NOT 走 Mac App Store 或 Microsoft Store。发行说明 MUST 来自 GitHub Releases 正文。

#### Scenario: 版本真实、检查不假装

- GIVEN 用户打开更新页
- WHEN 用户点检查更新且没有可用发行物
- THEN 当前版本与本机构建一致
- AND 说明没有可用更新
- AND 不声称已经连上应用商店

#### Scenario: 检查失败不写成没有更新

- GIVEN 用户打开更新页
- WHEN 检查请求失败
- THEN 说明检查失败
- AND 不说明没有可用更新
- AND 不声称已经连上应用商店

#### Scenario: 仅更高有效版本可升级

- GIVEN 返回列表含乱序版本、草稿、无效标签或仅构建元数据不同的版本
- WHEN 检查更新或准备安装
- THEN 按语义版本选择指定通道中最高的非草稿有效版本，不依赖返回顺序
- AND 只有版本优先级高于当前版本才可升级；同版和旧版不得触发下载

#### Scenario: 自动下载按通道排队安装

- GIVEN 自动下载已打开且当前通道有包
- WHEN 用户点检查更新
- THEN 通过 updater 排队安装
- AND 展示该通道发行说明
- AND 不声称已经从应用商店安装

## 测试映射

| 场景 | 测试 |
|---|---|
| 打开设置 | `WorkbenchShell.spec.js` opens settings from the sidebar |
| 保存反馈、未保存退出与危险确认 | `SettingsInteraction.spec.js` 写入失败回滚、待保存禁止关闭、草稿保留与取消/确认边界 |
| 简单选项与复杂表单 | `SettingsLayout.spec.js` 单一标题、当前页标识、纵向资料表单与切页保留草稿；浏览器十页尺寸/滚动检查 |
| 未实现页 | `WorkbenchShell.spec.js` labels the proxy row as follow-system instead of available |
| 空代理跟随系统 | `WorkbenchShell.spec.js` labels the proxy row as follow-system instead of available；`httpProxy.test.js` treats a blank value as follow-system；`desktop/src-tauri` `empty_setting_follows_system` |
| 填写后本机走代理 | `WorkbenchShell.spec.js` persists a manual http proxy from the settings row；`httpProxy.test.js` accepts http and https proxy urls；`desktop/src-tauri` `http_and_https_urls_are_accepted` |
| 非法代理不保存 | `WorkbenchShell.spec.js` rejects an invalid proxy url without saving；`httpProxy.test.js` rejects socks, other schemes, and garbage；`desktop/src-tauri` `other_schemes_and_garbage_are_rejected` |
| 十类都在 | `WorkbenchShell.spec.js` lists ten settings categories |
| 缺少的页不得消失 | `WorkbenchShell.spec.js` keeps the updates page without claiming a store check |
| 开机启动 | `desktopPrefs.test.js` persists launch at login on macos / windows / linux；`WorkbenchShell.spec.js` saves launch at login on macos / windows without claiming nsis / linux without claiming release qa；`windowChrome.test.js` treats Linux as linux |
| 保存快捷键 | `ShortcutInput.spec.js` 物理组合、输入过滤、Tab/Esc；`SettingsInteraction.spec.js` 三项录入保存和重复冲突；`shortcut.test.js` 录制期间回调保护 |
| 注册快捷键 | `shortcut.test.js` does not persist when register throws |
| 新建与粘贴快捷键可见 | `WorkbenchShell.spec.js` shows new and paste shortcut rows |
| 导入预览 | `desktop/src-tauri` `import_preview_does_not_write`；`library.test.js` previews import without writing |
| 备份恢复 | `desktop/src-tauri` `restore_replaces_library`、`failed_restore_leaves_library`；`library.test.js` rejects sqlite file backup in the browser memory library |
| 打开目录与 ZIP 行可见 | `WorkbenchShell.spec.js` shows open directory and zip rows with existing backup actions；`library.test.js` exports a zip payload without dropping memory prompts；`desktop/src-tauri` `export_zip_does_not_remove_sqlite`、`auto_backup_leaves_existing_backup` |
| 切换主题 | `desktop/src-tauri` `theme_persists_as_dark` |
| 外观增加项可见 | `WorkbenchShell.spec.js` shows appearance extras including follow-system theme |
| 界面语言切换壳层文案 | `WorkbenchShell.spec.js` switches chrome copy to English and back；`uiStrings.test.js` switches chrome labels between Chinese and English |
| 当前账号接已有登录 | `WorkbenchShell.spec.js` shows the current account from the existing login |
| 下载保留作者 | `WorkbenchShell.spec.js` keeps author on download when the setting is on；`square.test.js` copies author onto the local row only when keep_author_on_download is on；`desktop/src-tauri` `keeps_author_on_downloaded_prompt_without_rewriting_content` |
| 看到我的发布 | `WorkbenchShell.spec.js` lists my pending publications on the account page；`backend` `lists_own_publications_and_hides_other_accounts` |
| 保存作者资料 | `WorkbenchShell.spec.js` saves author display name after login and refuses when signed out；`backend` `saves_display_name_for_signed_in_user_and_rejects_anonymous` |
| 同步行可见且不假装 | `WorkbenchShell.spec.js` shows sync rows without requesting the backend |
| 冲突处理标明较新者胜 | `WorkbenchShell.spec.js` labels conflict handling as newer-wins instead of unavailable |
| 冲突处理可选保留本地 | `WorkbenchShell.spec.js` keeps the local body when keep-local is selected before syncing；`librarySync.test.js` keeps the local body when keep-local is chosen and remote updated_at is newer |
| 仅 Wi-Fi 同步图片可开关 | `WorkbenchShell.spec.js` persists wifi-only image sync from the settings row；`WorkbenchShell.spec.js` does not claim Wi-Fi image sync is missing because there is no cloud engine |
| 自动同步收藏可开关 | `WorkbenchShell.spec.js` persists auto-sync queue from the settings row；`WorkbenchShell.spec.js` opens settings from the sidebar；`WorkbenchShell.spec.js` shows sync rows without requesting the backend |
| 模型页可见且不外传正文 | `WorkbenchShell.spec.js` shows model rows without sending prompt bodies；`UsePromptModal.spec.js` shows a local variable hint when hints are enabled；`variableHints.test.js` returns a local example for a known name and nothing for unknown |
| 显示模型标签接到卡片 | `WorkbenchShell.spec.js` shows model tags on cards when the setting is on；`WorkbenchShell.spec.js` hides model tags when the setting is off |
| 默认模型进入新建编辑器 | `WorkbenchShell.spec.js` preselects the default model in the editor；`library.test.js` persists the selected model on create and update |
| 关闭广场访问 | `WorkbenchShell.spec.js` does not request square when access is off |
| 同步状态不写没有云同步 | `WorkbenchShell.spec.js` does not claim the network page has no cloud sync；`WorkbenchShell.spec.js` does not claim Wi-Fi image sync is missing because there is no cloud engine |
| 清除使用历史不删正文 | `WorkbenchShell.spec.js` clears use history without deleting prompt content；`library.test.js` clears use counts without deleting prompt content；`desktop/src-tauri` `clear_use_history_keeps_prompt_content` |
| 匿名下载统计默关且可打开 | `WorkbenchShell.spec.js` does not claim anonymous download stats is unavailable；`WorkbenchShell.spec.js` persists anonymous download stats from the settings row |
| 打开后成功下载只上报条目 id | `square.test.js` posts anonymous download stats without auth after a successful download when the setting is on；`WorkbenchShell.spec.js` records anonymous download stats after a successful download when the setting is on |
| 关闭时不请求统计 | `square.test.js` does not post download stats when the setting is off；`WorkbenchShell.spec.js` does not record anonymous download stats when the setting is off；`square.test.js` keeps the local download when stats post fails |
| 浏览器预览不写本机钥匙串 | `WorkbenchShell.spec.js` does not claim the keychain row uses the local keychain in browser preview；`WorkbenchShell.spec.js` does not claim login writes refresh to the system keychain in browser preview |
| Tauri 标明本机钥匙串 | `WorkbenchShell.spec.js` labels the keychain row as local keychain inside Tauri |
| 版本真实、检查不假装 | `WorkbenchShell.spec.js` keeps the updates page without claiming a store check；`packageIsolation.test.js` keeps package version aligned with tauri and cargo；`updates.test.js` reports no update when the latest stable tag matches the tauri build；`updates.test.js` asks GitHub Releases and reports none when the list is empty |
| 检查失败不写成没有更新 | `WorkbenchShell.spec.js` does not treat a failed update check as no updates；`updates.test.js` does not treat a failed GitHub read as no updates |
| 仅更高有效版本可升级 | `updates.test.js` 旧版/同版构建、乱序草稿与预发行排序；Rust `commands::updates::tests` |
| 自动下载按通道排队安装 | `WorkbenchShell.spec.js` queues an updater install when auto-download is on and the channel has a package；`updates.test.js` queues an updater install when auto-download is on and the channel has a package |
