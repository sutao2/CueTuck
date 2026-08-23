# 设置

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M2 实现本机项 |
| 来源 | 原型设置弹窗信息架构，行为以本机为准 |

## Purpose

用弹窗管理本机偏好。第一期只落地有真实行为的项。

## Requirements

### Requirement: 入口

系统 MUST 从侧栏底部打开设置。设置 MUST 是工作台弹窗，不是旧产品那套独立设置路由站。

#### Scenario: 打开设置

- GIVEN 工作台已显示
- WHEN 用户点击「设置」
- THEN 出现设置弹窗
- AND 左侧为分类，右侧为当前页

### Requirement: 第一期页面

第一期 MUST 实现：常规、快捷键、数据与备份、外观。账号与广场、同步、网络与代理、更新可以显示「将在联网版提供」，MUST NOT 假装已接通。

#### Scenario: 未实现页

- GIVEN 第一期构建
- WHEN 用户打开「同步」
- THEN 页面说明尚未提供云同步
- AND 不调用后端

### Requirement: 快捷键

系统 MUST 能记录并保存启动器全局快捷键。与系统冲突时 MUST 提示失败，不得静默无效。

#### Scenario: 保存快捷键

- GIVEN 用户在快捷键页录制新组合
- WHEN 系统注册成功
- THEN 该组合能唤起启动器
- AND 旧组合不再唤起

### Requirement: 数据

系统 MUST 提供 JSON 导出、带预览的 JSON 导入、备份与恢复。导入 MUST 先预览再写入。失败 MUST 可取消且不写半份数据。

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

### Requirement: 外观

系统 MUST 支持浅色/深色，并持久化到 `settings` 表。启动器 MUST 读取同一主题键。

#### Scenario: 切换主题

- GIVEN 用户在外观页选择深色
- WHEN 系统保存
- THEN 再次读取 theme 为 `dark`
- AND 启动器读取同一键

## 测试映射

| 场景 | 测试 |
|---|---|
| 打开设置 | `WorkbenchShell.spec.js` opens settings from the sidebar |
| 未实现页 | 同上，打开「同步」见 `settings-unavailable` |
| 保存快捷键 | `shortcut.test.js` does not persist when register throws |
| 导入预览 | `desktop/src-tauri` `import_preview_does_not_write`；`library.test.js` previews import without writing |
| 备份恢复 | `desktop/src-tauri` `restore_replaces_library`、`failed_restore_leaves_library`；`library.test.js` rejects sqlite file backup in the browser memory library |
| 外观 | `desktop/src-tauri` `theme_persists_as_dark` |
