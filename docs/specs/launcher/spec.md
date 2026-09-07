# 启动器

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M3 实现唤起与搜索粘贴；M8 增加新建与粘贴最近使用快捷键 |
| 来源 | 旧 `prompt-launcher` 独立窗口，不是原型覆盖层 |
| 关联 | [ADR 0002](../../architecture/decisions/0002-preserve-current-launcher.md) |

## Purpose

用全局快捷键唤起独立窗口，搜索本地提示词，填写变量，复制或粘贴到先前的活动应用。

## Requirements

### Requirement: 独立窗口

系统 MUST 使用独立 Tauri 窗口承载启动器，不得用主窗口 modal 或全屏覆盖层代替。

#### Scenario: 快捷键唤起

- GIVEN 应用已运行且本地库已就绪
- WHEN 用户按下已配置的全局快捷键
- THEN 启动器窗口显示并聚焦搜索框
- AND 在当前显示器工作区横向居中，顶部位于扣除收起高度后可用纵向空间的三分之一处；坐标按显示器缩放计算
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

M3 已交付的唤起快捷键 MUST 保留。M8 起系统 MUST 另支持：新建提示词（打开本机新建）、快速粘贴最近使用（粘贴上一条已完成变量替换的提示词）。冲突时 MUST 提示失败，不得静默无效。启动器仍 MUST NOT 请求广场或管理接口。

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

### Requirement: 粘贴到活动应用

填写页 MUST 明确区分复制与粘贴，默认 Enter 只复制，不向外部应用注入按键。macOS 原生启动器的复制 MUST 使用系统剪贴板桥接并检查写入结果；浏览器仅使用自身剪贴板能力。无正文不得执行空复制或覆盖最近文本。

#### Scenario: 占位符逐项输入

- GIVEN 已选择带多个变量的提示词
- WHEN 进入填写并按 Enter
- THEN 首个变量自动聚焦，Enter 前进到下一变量，最后一个变量 Enter 复制最终文本；Ctrl/Cmd+Enter 可从任一变量复制
- AND Shift+Enter 换行，Tab/Shift+Tab 保留正常导航，输入法确认文字不触发复制
- AND 重复变量只填写一次，未填变量在预览中保留；无变量时正文占整栏且复制按钮聚焦

#### Scenario: 返回与再唤起

- GIVEN 正在填写或搜索，用户点击返回、关闭、失焦隐藏或再次唤起
- WHEN 状态切换完成
- THEN 返回保留查询及高亮并聚焦搜索；关闭或失焦后的再次唤起清空旧填写并聚焦空搜索
- AND 每次唤起重读主题偏好；复制/粘贴进行中不得因失焦丢失待完成文本或重复执行

#### Scenario: 不关闭偏好与失败

- GIVEN 用户关闭使用后自动关闭，或复制/粘贴失败
- WHEN 本次操作完成
- THEN 复制成功保持填写与成功反馈；粘贴需要隐藏并归还焦点，成功后按偏好决定是否返回启动器
- AND 写剪贴板失败不记使用；仅粘贴失败保留已复制文本、重新显示填写及明确降级原因
- AND 使用记录失败不声称复制失败，不自动清空填写

系统 MUST 先把文本写入剪贴板，再隐藏启动器并把焦点交还原应用，然后模拟粘贴。失败时 MUST 明确降级为仅复制，不得假装已粘贴。

macOS MUST 连续确认原窗口焦点稳定后再发送按键；超时、目标退出或辅助功能权限不足必须停止。Windows/Linux 暂无经过核实的原窗口恢复实现，不自动注入按键，明确降级为已复制后手动粘贴，不宣称跨平台原生验收通过。

#### Scenario: 粘贴成功

- GIVEN 填写完成且目标应用仍可聚焦
- WHEN 用户选择粘贴到原窗口
- THEN 目标应用收到粘贴
- AND 按「使用后关闭」偏好隐藏或恢复启动器；反馈只声明已发送指令，不将系统按键发送成功等同于外部应用已接收

#### Scenario: 粘贴失败

- GIVEN 系统拒绝模拟按键或焦点未回到目标应用
- WHEN 粘贴命令失败
- THEN 剪贴板仍保留最终文本
- AND 用户看到「已复制，未能粘贴」类反馈

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
| macOS 启动器窗口 | `LauncherApp.spec.js` uses mac chrome on macos；空查询 `is-collapsed`；`launcherWindow.test.js` sizes the palette like the old independent window；`palette_heights_match_old_window` |
| 1 万条查询预算 | `./scripts/launcher-search-bench`（`search_ten_thousand_prompts_bench`，release，不作为 CI 红灯） |
