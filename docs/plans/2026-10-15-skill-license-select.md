# Skill 发布许可选择

## 计划

1. 发布表单复用可搜索下拉框，常用许可包含简要说明；选中后显示说明与条款链接。
2. 保留包内已有许可原值；无许可不自动替用户授权，用户选择后按既有字段提交。创建表单保持原样。
3. 回归选择、已有非预置许可、空许可和提交，构建及文档检查通过后提交。

## 场景

- Given 待发布 Skill When 选择许可 Then 每项可见简述，选中后显示说明及条款链接，提交使用许可值而非说明。
- Given 已有非预置许可 When 打开发布页及切换后返回 Then 原许可仍可选择，不被静默替换。
- Given 未声明许可 When 进入发布页 Then 显示选择提示，不默认赋予 MIT；选择并确认后可提交。

## 依据

说明核对 [Choose a License](https://choosealicense.com/licenses/)、[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) 与 [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/)。简述不替代完整条款。

## 状态

已随 [beta.32](2026-10-15-beta32-release.md) 发布。

SkillsPage、SkillCommunity 与 SearchableSelect 共 41 项测试通过，桌面生产构建、docs-check 和 diff 检查通过。覆盖空许可不自动授权、五种选项及说明、非预置原许可切换保留、许可变更重置公开确认、实际提交值与文件包保持。中英文文案已补齐，未进行双平台原生窗口实测。
