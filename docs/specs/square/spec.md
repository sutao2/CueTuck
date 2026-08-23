# 广场

| 字段 | 值 |
|---|---|
| 状态 | 实现中 |
| 第一期 | 不实现远端行为 |

## Purpose

联网后浏览社区提示词与合集。下载不要求登录；收藏与发布要求登录。

## Requirements

### Requirement: 第一期不可用

第一期构建 MUST NOT 请求广场 API。用户进入广场空间时 MUST 看到未开放说明。

#### Scenario: M2 构建无广场请求

- GIVEN 第一期桌面构建
- WHEN 用户打开工作台或点击广场
- THEN 不出现成功的广场列表请求
- AND 本地空间仍可用

### Requirement: 浏览（M5）

系统 MUST 在联网时展示推荐、最新、热门、收藏（已登录）与模型筛选。合集与提示词在内容区混排，合集不进分类树。

#### Scenario: 离线

- GIVEN M5 已实现广场且设备离线
- WHEN 用户停留在广场
- THEN 显示非阻断离线说明
- AND 提供前往本地的入口

### Requirement: 下载与收藏分离（M5）

「下载」MUST 把远端记录复制到本地 SQLite，不要求登录。「收藏」MUST 是账号关系，未登录时打开登录并说明原因。

#### Scenario: 未登录下载

- GIVEN 用户未登录且网络可用
- WHEN 用户下载一条广场提示词
- THEN 本地库新增一条 `source=downloaded` 的副本
- AND 不弹出登录

#### Scenario: 未登录收藏

- GIVEN 用户未登录
- WHEN 用户点击收藏
- THEN 出现登录提示且原因包含「收藏」
- AND 本地库不因此新增副本

## 测试映射

| 场景 | 测试 |
|---|---|
| M2 构建无广场请求 | 已由 M5 浏览替代；离线不阻断本地 |
| 离线 | `WorkbenchShell.spec.js` shows a non-blocking offline notice and can return to local；`LauncherApp.spec.js` does not request square while searching locally |
| 未登录下载 | 未开始 |
| 未登录收藏 | 未开始 |
| 合同 path 与匿名下载 | `squareContract.test.js` lists every contract path |
| 浏览混排 | `WorkbenchShell.spec.js` shows square items in the content grid not the category tree；`backend` `lists_square_items_without_login` |
