# 启动器

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M3 实现 |
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
- AND 主窗口不必被提到前台

#### Scenario: 关闭

- GIVEN 启动器可见
- WHEN 用户按 Esc 或点击关闭
- THEN 窗口隐藏
- AND 搜索态恢复为空查询

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

系统 MUST 支持方向键移动高亮，Enter 打开填写或预览，修饰键+Enter 直接复制当前条。

#### Scenario: Enter 填写

- GIVEN 高亮一条含 `{{变量}}` 的提示词
- WHEN 用户按 Enter
- THEN 进入填写态并展示该提示词的变量字段

#### Scenario: 直接复制

- GIVEN 高亮一条提示词
- WHEN 用户按 Ctrl+Enter 或 Cmd+Enter
- THEN 当前模板按已填或空变量渲染后写入剪贴板
- AND 启动器隐藏

### Requirement: 粘贴到活动应用

系统 MUST 先把文本写入剪贴板，再隐藏启动器并把焦点交还原应用，然后模拟粘贴。失败时 MUST 明确降级为仅复制，不得假装已粘贴。

#### Scenario: 粘贴成功

- GIVEN 填写完成且目标应用仍可聚焦
- WHEN 用户选择粘贴到当前窗口
- THEN 目标应用收到粘贴
- AND 启动器隐藏

#### Scenario: 粘贴失败

- GIVEN 系统拒绝模拟按键或焦点未回到目标应用
- WHEN 粘贴命令失败
- THEN 剪贴板仍保留最终文本
- AND 用户看到「已复制，未能粘贴」类反馈

### Requirement: 选中文本

系统 MUST 仅在 macOS 提供读取选中文本并填入变量的能力。其他平台 MUST 不展示该入口。

#### Scenario: 非 macOS

- GIVEN 运行平台不是 macOS
- WHEN 用户打开启动器填写态
- THEN 界面不出现「读取选中文本」能力

### Requirement: 失焦

系统 MUST 在失焦后隐藏启动器，但唤起后的短保护期内不得因系统抢焦而闪关。保护期行为以旧 `launcher.rs` 为准。
