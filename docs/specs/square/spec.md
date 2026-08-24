# 广场

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M5 实现 |
| 第一期 | M0–M4 不实现远端行为 |

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

#### Scenario: 条目详情

- GIVEN 广场有 id 为 `sq-1` 的条目
- WHEN 匿名请求 `GET /v1/square/items/sq-1`
- THEN 返回 200 且含标题
- AND 请求不存在的 id 返回 404

#### Scenario: 浏览排序与模型筛选

- GIVEN 预发种子含不同标题与模型
- WHEN 分别请求 `sort=recommended`、`latest`、`hot`
- THEN 三种顺序可区分
- AND `model` 查询只返回该模型
- AND 不得声称这是生产热度算法

#### Scenario: 已登录收藏排序

- GIVEN 用户已登录并收藏了某条
- WHEN 请求 `sort=favorites` 且携带 Access
- THEN 列表含该条
- AND 未登录时收藏排序为空列表

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

#### Scenario: 已登录收藏

- GIVEN 用户已登录且网络可用
- WHEN 用户收藏一条广场条目
- THEN 账号收藏关系被写入
- AND 本地库不因此新增 `source=downloaded` 副本

#### Scenario: 取消收藏

- GIVEN 用户已登录且已收藏该条
- WHEN 用户取消收藏
- THEN 账号收藏关系删除
- AND 本地已下载副本仍在

## 测试映射

| 场景 | 测试 |
|---|---|
| M2 构建无广场请求 | 已由 M5 浏览替代；离线不阻断本地 |
| 离线 | `WorkbenchShell.spec.js` shows a non-blocking offline notice and can return to local；`LauncherApp.spec.js` does not request square while searching locally |
| 未登录下载 | `WorkbenchShell.spec.js` downloads a square prompt without login as source=downloaded；`square.test.js` writes a local copy with source=downloaded；`imports_downloaded_prompt_with_source`；`serves_square_item_content_without_login` |
| 未登录收藏 | `WorkbenchShell.spec.js` opens login from favorite without writing a local copy |
| 已登录收藏 | `WorkbenchShell.spec.js` favorites a square item while logged in without writing a local copy；`square.test.js` puts a favorite without writing a local copy；`backend` `put_favorite_lists_for_account` |
| 取消收藏 | `WorkbenchShell.spec.js` keeps a downloaded copy after unfavorite；`backend` `delete_favorite_removes_account_relation` |
| 合同 path 与匿名下载 | `squareContract.test.js` lists every contract path |
| 浏览混排 | `WorkbenchShell.spec.js` shows square items in the content grid not the category tree；`backend` `lists_square_items_without_login` |
| 条目详情 | `backend` `serves_square_item_without_login` |
| 浏览排序与模型筛选 | `backend` `sorts_recommended_latest_and_hot_apart` |
| 已登录收藏排序 | `backend` `favorites_sort_requires_login` |
| 进程重启后列表仍在 | `backend` `publication_favorite_and_settings_survive_postgres` |
