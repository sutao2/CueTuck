# 启动器

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M3 实现唤起与搜索粘贴；M8 增加新建与粘贴最近使用快捷键 |
| 来源 | 旧 `prompt-launcher` 独立窗口，不是原型覆盖层 |
| 关联 | [ADR 0002](../../architecture/decisions/0002-preserve-current-launcher.md) |

## Purpose

用全局快捷键唤起独立窗口，搜索本地提示词，填写变量并复制最终文本。

## Requirements

### Requirement: 独立窗口

系统 MUST 使用独立 Tauri 窗口承载启动器，不得用主窗口 modal 或全屏覆盖层代替。

滚动区域外观遵循[工作台的桌面共用规则](../workbench/spec.md)；变量表单、结果和预览分别保留自己的滚动区域，不让变量滚动条压住字段边缘。

### Requirement: 本机启动器偏好

`launcher_preferences` JSON 单键保存四项：`size` 为 compact（默认 620×420）、standard（680×500）、large（760×560）；`position` 为 upper（默认）或 center；`fontSize` 为 12（默认）、14、16；`resultLimit` 为 10、20（默认）、50。损坏 JSON 和非法字段逐项回退默认，启动器初始化和显式重新唤起时读取，进行中不切换填写偏好。字号仅影响变量输入和正文预览，不改变主窗口；空搜索高度仍为 64。原生尺寸不超过显示器工作区。

#### Scenario: 未设置偏好

- GIVEN 新安装或旧库中没有 `launcher_preferences` 记录
- WHEN 通过按钮或全局快捷键唤起启动器
- THEN 原生读取使用默认偏好，窗口正常显示，不强制用户先保存设置
- AND 不新增或覆盖设置记录；真实数据库错误仍返回失败。修复与验收见[缺省偏好计划](../../plans/2026-09-12-launcher-defaults.md)

#### Scenario: 偏好消费

- GIVEN 已保存大小、位置、字号与结果数量
- WHEN 下次唤起启动器
- THEN 窗口大小和定位、输入/预览字号、搜索结果上限使用已保存值
- AND 搜索与填写同尺寸、切换不重新定位，浏览器入口采用所选大小但不保证系统窗口位置

#### Scenario: 快捷键唤起

- GIVEN 应用已运行且本地库已就绪
- WHEN 用户按下已配置的全局快捷键
- THEN 无进行中填写时显示并聚焦搜索框；失焦暂存的填写按下述恢复规则继续
- AND 在当前显示器工作区横向居中，默认顶部位于扣除展开高度后剩余纵向空间的四分之一处；选择居中则为二分之一处。坐标按显示器缩放计算，剩余空间不足时从工作区顶部展开
- AND 主窗口不必被提到前台

#### Scenario: 关闭

- GIVEN 启动器可见
- WHEN 用户按 Esc 或点击关闭
- THEN 窗口隐藏
- AND 搜索态恢复为空查询

#### Scenario: macOS 启动器窗口

- GIVEN 用户在 macOS 唤起启动器
- WHEN 独立窗口显示
- THEN 它是旧产品那种无框透明调色板：空查询收成一条搜索栏，有结果后窗口增高
- AND 快捷键记号用 Mac 符号，不写 `Ctrl Space`
- AND 仍是 label `launcher` 的独立窗口，不是主窗口覆盖层，也不画主窗口那种 Overlay 红绿灯

#### Scenario: 输入与清空时位置稳定

- GIVEN 启动器已唤起，用户可能已拖动窗口
- WHEN 用户反复输入、清空查询，或进入和退出变量填写
- THEN 只调整窗口高度，窗口左上角及搜索栏位置不随布局切换移动
- AND 仅重新唤起窗口时恢复默认偏上位置，不在内容尺寸变化时重新定位

### Requirement: 附加全局快捷键

M3 已交付的唤起快捷键 MUST 保留。M8 起系统 MUST 另支持：新建提示词（打开本机新建）、快速粘贴最近使用（粘贴上一条已完成变量替换的提示词）。冲突时 MUST 提示失败，不得静默无效。启动器默认搜索 MUST NOT 请求广场；显式联网边界遵循 [ADR 0023](../../architecture/decisions/0023-launcher-explicit-actions.md)，始终禁止管理接口。

#### Scenario: 新建提示词快捷键

- GIVEN 应用已运行且该快捷键已接通
- WHEN 用户按下已保存的新建提示词组合
- THEN 打开本机新建提示词
- AND 不请求广场

#### Scenario: 粘贴最近使用快捷键

- GIVEN 用户刚完成一次变量替换且该快捷键已接通
- WHEN 用户按下已保存的快速粘贴组合
- THEN 上一条完成替换的文本被粘贴或明确降级为复制
- AND 不请求广场
- AND 启动器未打开时重新记录当前目标，不使用上次唤起的旧目标；没有最近文本或系统拒绝时显示启动器及具体错误

### Requirement: 本地即时搜索

系统 MUST 只依赖本地数据返回第一批结果，不得为展示本地结果等待网络。1 万条提示词下单次查询 MUST 小于 50ms。

#### Scenario: 输入即搜

- GIVEN 本地存在标题含「官网」的提示词
- WHEN 用户输入「官网」
- THEN 该提示词出现在结果中
- AND 结果在本地查询完成后立即渲染

#### Scenario: 空查询

- GIVEN 启动器在搜索态且查询为空
- WHEN 窗口显示
- THEN 不展示结果列表或仅保持收起布局
- AND 不发起广场请求

### Requirement: 键盘选择

搜索加载中与失败 MUST 区别于无结果。高亮变化 MUST 滚动到可见范围并更新 combobox 的活动选项。鼠标点击和键盘 Enter MUST 进入相同流程；输入法组字中的 Enter/Escape MUST NOT 触发使用或关闭，长按 Enter MUST NOT 连续执行。

系统 MUST 支持方向键移动高亮，Enter 打开填写或预览，修饰键+Enter 直接复制当前条。

#### Scenario: Enter 填写

- GIVEN 高亮一条含 `{{变量}}` 的提示词
- WHEN 用户按 Enter
- THEN 进入填写态并展示该提示词的变量字段

#### Scenario: 无变量预览与复制失败

- GIVEN 高亮提示词没有变量
- WHEN 按 Enter 后复制被剪贴板拒绝
- THEN 先进入正文预览，失败后显示错误且保留预览，不记录使用、不更新最近粘贴文本、不关闭窗口
- AND 重试成功后才记录使用；Ctrl/Cmd+Enter 仍允许直接复制

#### Scenario: 搜索响应逆序

- GIVEN 用户连续输入不同搜索词或清空输入
- WHEN 先前查询较晚返回
- THEN 不覆盖当前搜索结果，也不重新打开已清空的列表

#### Scenario: 直接复制

- GIVEN 高亮一条提示词
- WHEN 用户按 Ctrl+Enter 或 Cmd+Enter
- THEN 当前模板按已填或空变量渲染后写入剪贴板
- AND 按「使用后关闭」偏好隐藏或保留搜索与成功反馈

### Requirement: 复制最终提示词

#### Scenario: 紧凑填写与完成后退场

- GIVEN 用户从其他应用唤起启动器并进入填写
- WHEN 展示 0、3 或超过一屏的变量
- THEN 填写窗与搜索结果窗使用相同的所选尺寸（默认紧凑），空查询沿用宽度并收为 64 高；来回切换不缩窗，紧凑双栏中字段和预览可独立滚动，底部操作始终可见；无变量时正文通栏。浏览器预览入口使用相同展开尺寸
- WHEN 回车复制成功且使用后关闭开启
- THEN 隐藏窗口后不再给内部 DOM 聚焦，迟到的 focus 事件不得唤回窗口；下次显式唤起才恢复聚焦
- AND macOS 在主动关闭且启动器仍聚焦时归还原应用，不主动展示无关主窗口；失焦关闭不抢回用户刚切换的应用，原目标退出也不重新弹出启动器

填写页仅提供最终正文复制，不展示粘贴到原窗口入口；Enter 只复制，不向外部应用注入按键。macOS 原生启动器的复制 MUST 使用系统剪贴板桥接并检查写入结果；浏览器仅使用自身剪贴板能力。无正文不得执行空复制或覆盖最近文本。

macOS 剪贴板子进程 MUST 显式使用 UTF-8，不依赖 GUI 启动时的 locale；退出码成功后仍需确认回读字节与待复制正文一致。回读失败或不一致视为复制失败，不关闭窗口或记录使用。

- Given GUI 环境缺少 UTF-8 locale When 回车复制中文、emoji 或多行正文 Then 关闭后可完整粘贴；空回读或内容不一致时保留预览并提示重试。

#### Scenario: 占位符逐项输入

- GIVEN 已选择带多个变量的提示词
- WHEN 进入填写并按 Enter
- THEN 首个变量自动聚焦，Enter 前进到下一变量，最后一个变量 Enter 复制最终文本；Ctrl/Cmd+Enter 可从任一变量复制
- AND Shift+Enter 换行，Tab/Shift+Tab 保留正常导航，输入法确认文字不触发复制
- AND 重复变量只填写一次，未填变量在预览中保留；无变量时正文占整栏且复制按钮聚焦

#### Scenario: 返回与再唤起

- GIVEN 正在填写或搜索，用户点击返回、关闭、失焦隐藏或再次唤起
- WHEN 状态切换完成
- THEN 返回保留查询及高亮并聚焦搜索；明确关闭后再次唤起清空旧填写并聚焦空搜索；失焦隐藏则保留填写/预览、已填值、当前字段与滚动，再唤起直接恢复填写尺寸
- AND 每次唤起重读主题偏好；复制进行中不得因失焦丢失待完成文本或重复执行
- AND 参数仅保留于进程内，不跨重启持久化；结束与异步隔离验收见[草稿恢复计划](../../plans/2026-09-12-launcher-draft-resume.md)

#### Scenario: 跟随系统主题

- GIVEN 主题偏好为 system
- WHEN 系统明暗主题变化
- THEN 启动器同步更新；显式 light/dark 不受系统切换覆盖，卸载时移除监听

#### Scenario: 不关闭偏好与失败

- GIVEN 用户关闭使用后自动关闭，或复制失败
- WHEN 本次操作完成
- THEN 保持填写与成功或失败反馈，不调用粘贴或恢复目标窗口
- AND 写剪贴板失败不记使用；使用记录失败不声称复制失败，不自动清空填写

输入操作与移除边界见 [ADR 0024](../../architecture/decisions/0024-launcher-copy-only.md)。共享原生粘贴规则仅用于上述独立全局快捷键。

### Requirement: 选中文本

系统 MUST 仅在 macOS 提供读取选中文本并填入变量的能力。其他平台 MUST 不展示该入口。

读取 MUST 由用户点击触发，只针对启动器唤起前记录的应用，填入当前选中的变量，而非无条件覆盖第一项。无选中文本或读取失败时保留旧输入并说明原因。浏览器预览无原生能力时不得展示无效入口。

#### Scenario: 读取到当前变量

- GIVEN macOS 原应用存在选中文本，用户正在填写第二个变量
- WHEN 点击读取选中文本
- THEN 将原应用选中文本填入第二项，其他变量不变
- AND 不读取启动器自身文本、不改剪贴板、不静默申请或绕过辅助功能权限

#### Scenario: 非 macOS

- GIVEN 运行平台不是 macOS
- WHEN 用户打开启动器填写态
- THEN 界面不出现「读取选中文本」能力

### Requirement: 失焦

系统 MUST 在失焦后隐藏启动器，但唤起后的短保护期内不得因系统抢焦而闪关。保护期行为以旧 `launcher.rs` 为准。

## 测试映射

| 场景 | 测试 |
|---|---|
| 偏好消费 | `LauncherInteraction.spec.js` 字号与结果上限、重新唤起刷新；`launcherPreferences.test.js` 浏览器尺寸；Rust `preferences_select_sizes_centering_and_fit_work_area`；验收见 `plans/2026-09-08-launcher-preferences.md` |
| 紧凑填写与完成后退场 | `LauncherInteraction.spec.js` 隐藏后不聚焦、初始隐藏与显式恢复；`launcherWindow.test.js` 紧凑高度；`palette_reserves_expanded_space_on_small_screens` 小屏定位；原生验收边界见 `plans/2026-09-07-launcher-compact-dismiss.md` |
| 独立窗口 label | `desktop/src/platform/launcherWindow.test.js`；`launcher_label_is_stable` |
| 空查询 | `LauncherApp.spec.js` hides results on empty query |
| 输入与清空时位置稳定 | `launcherWindow.test.js` 原生命令保留左上角、仅唤起时定位的源码合同；`LauncherApp.spec.js` 连续输入/清空和填写返回的布局调用回归（非原生坐标实测） |
| 输入即搜 | `LauncherApp.spec.js` lists a local title hit |
| Enter 填写 | `LauncherApp.spec.js` opens fill step when Enter hits a variable prompt |
| 直接复制 | `launcherKeyboard.test.js` activates default on Enter and copy on Ctrl+Enter |
| 快捷键唤起 | `shortcut.test.js` persists after a successful register；`palette_opens_above_center_on_scaled_and_offset_monitors` 定位计算 |
| 新建提示词快捷键 | `shortcut.test.js` does not persist when an extra shortcut register throws；`open_new_prompt` 打开主窗口 |
| 粘贴最近使用快捷键 | 同上 extras；启动器写入 `last_rendered_prompt` 后 `paste_recent_prompt` 粘贴 |
| 关闭 / 失焦 | `desktop/src-tauri` `focus_grace_is_600ms`；Esc 走 `resetAndHide` |
| 粘贴成功 / 粘贴失败 | `paste.test.js` keeps clipboard text when paste command fails |
| 非 macOS | `selectedText.test.js` hides selected-text on windows |
| 多变量回车/多行/输入法/重复按键/特殊名称/焦点恢复/空复制 | `LauncherInteraction.spec.js` |
| 使用中保护/关闭偏好/原生生命周期/读取当前变量/失败保留 | `LauncherInteraction.spec.js`（原生接口替身） |
| 粘贴前稳定焦点与超时 | `focus_must_be_stable_before_pasting`；`unready_target_times_out_without_proceeding` |
| macOS 启动器窗口 | `LauncherApp.spec.js` uses mac chrome on macos；空查询 `is-collapsed`；`launcherWindow.test.js` keeps search and fill equally compact、配置/浏览器入口一致；`palette_heights_keep_search_and_fill_compact`；本轮验收见 `plans/2026-09-08-launcher-refinement.md` |
| 跟随系统主题 | `LauncherInteraction.spec.js` system 实时切换、显式偏好优先与监听释放 |
| 1 万条查询预算 | `./scripts/launcher-search-bench`（`search_ten_thousand_prompts_bench`，release，不作为 CI 红灯） |

### Requirement: 输入快捷操作

非空输入 MUST 仅提供创建提示词、AI 优化、广场搜索，支持鼠标和键盘。创建仅在显式保存时写库；优化保留原文、展示可编辑结果，由用户采用或保存。无配置提供本机 AI 设置入口；失败不能覆盖原文或伪造结果。广场查询仅显式切换后启用，250ms 合并并取消旧请求，遵循广场访问开关；选择结果打开工作台详情，不自动下载/投稿。场景与验收见[交互计划](../../plans/2026-09-13-interactions-launcher-ai.md)。
