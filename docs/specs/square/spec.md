# 广场

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M5 实现 |
| 第一期 | M0–M4 不实现远端行为 |

## Purpose

联网后浏览社区提示词与合集。下载不要求登录；收藏与发布要求登录。

## Requirements

### Requirement: 下载与收藏数据展示

分页广场 MUST 返回每条的 `download_count` 和 `favorite_count`，分别表示已记录匿名下载次数与当前收藏账号数。客户端网格/列表展示数值，热门明确说明按已记录下载量排序；未返回字段显示「—」。不得把下载次数描述为独立人数或全量下载。统计开关和采集行为保持不变，客户端不因本地下载或离线收藏排队而虚增服务端统计。

我的发布 MUST 在本人鉴权下返回各作品计数，页面汇总投稿数、记录下载次数与当前收藏次数；后者是各作品收藏人数相加，不是去重粉丝数。提供最新提交、下载最多、收藏最多排序。下架保留已有数据，从未上架计数为零；错误和缺失字段不伪装零。

#### Scenario: 统计口径与本人隔离

- GIVEN 多条作品有不同下载记录和收藏关系
- WHEN 查看热门或我的发布
- THEN 广场按既有热门规则排序，个人页只统计本人作品并支持按两项数据排序
- AND 重复收藏不增加人数、取消后减少，不暴露收藏者邮箱，未上报下载不计数
- AND 卡片与汇总保持缺失/错误状态，不把模拟或缺省数值当作真实统计

本轮页面交互以[工作台优化计划](../../plans/2026-09-12-workbench-usability.md)对应场景为准；替代卡片直接编辑、已下载按钮禁用和生产详情排版示例入口。

### Requirement: 清空后不重建演示内容

后端运行设置 `square_seed_disabled=true` MUST 阻止空广场启动时自动写入演示条目；已有内容不得因该标志改写。缺省保留现有开发初始化行为，数据库读取失败不得执行种子写入。

#### Scenario: 已清空实例重启

- GIVEN 维护操作已备份并清空内容，同时设置持久化禁用种子标志
- WHEN 重启后端
- THEN 广场保持为空，账号和其他配置保留

### Requirement: 智能体有界检索

独立只读分页接口 MUST 按标题/id 稳定排序、过滤非 online 条目，支持标题/摘要字面关键词、分类含子级、模型及有界 limit/offset；禁止匿名时返回 401，不改变现有工作台列表响应。不得读取本地库或记下载统计，见 [P2 计划](../../plans/2026-09-08-mcp-square.md)。

### Requirement: 公开附件下载

单条提示词详情 MUST 展示已批准附件清单，显式查看时联网读取；下载 MUST 先完成全部文件大小、类型和 SHA-256 校验，再与正文原子导入，新本地附件使用新 UUID。失败不留下部分提示词，重复下载沿用远端 ID 防重。未批准、下架、回收站及匿名策略约束见 [P1c 计划](../../plans/2026-09-08-publication-assets.md)。

### Requirement: 第一期不可用

第一期构建 MUST NOT 请求广场 API。用户进入广场空间时 MUST 看到未开放说明。

#### Scenario: M2 构建无广场请求

- GIVEN 第一期桌面构建
- WHEN 用户打开工作台或点击广场
- THEN 不出现成功的广场列表请求
- AND 本地空间仍可用

### Requirement: 浏览（M5）

批量导入 MUST 不向正文追加来源或许可尾注；来源、许可及筛选证据单独保存。桌面广场 MUST 使用服务端分页，每批最多 48 条，滚动按需加载与虚拟化挂载；保留完整匹配总数，筛选变更取消旧请求并重置列表，续页失败保留内容供显式重试。列表不读取或传输正文/成员，只传摘要和单张封面。兼容接口边界与场景见[性能计划](../../plans/2026-09-09-square-performance.md)。获取、去重、安全评分与本机幂等导入场景见[批量筛选计划](../../plans/2026-09-08-prompt-corpus.md)。自动筛选不得标称逐条人工审核或保证生成效果。

公开数据包 MUST 支持独立校验与重复追加不增副本；同 ID 正文发生冲突时整批拒绝，不覆盖现有编辑。缺少参考图或未核实模型时 MUST 保持为空，不用随机图或猜测模型补齐。

非图片公开文本导入 MUST 提取独立 Prompt 段落，排除仓库说明及独立示例输出；MIT 许可全文与来源路径保存在独立元数据，不追加到正文。分类优先采用明确用途标题及来源目录，不能因正文偶然提到代码、视频等而误分类。依赖工作区数据或 Copilot 的模板 MUST 标注使用条件；来源、筛选与验收见[非图片补充计划](../../plans/2026-09-09-text-corpus.md)。

公开条目可附带 `reference` 来源元数据（作者、来源 URL、许可、参考图片 URL）；旧记录缺省为空。客户端 MUST 仅加载受信 HTTPS 图片地址，不执行来源文本；显示参考图署名许可，失败仍可阅读正文。样本导入与验收见[开源样本计划](../../plans/2026-09-08-community-import.md)。

详情页 MUST 仅展示当前内容的参考图；无参考图时直接展示正文，不提供无关摄影排版示例入口。来源署名与下载附件边界保持不变，见[工作台优化计划](../../plans/2026-09-12-workbench-usability.md)。

广场 MUST 不显示未查询排序的假零计数；结果标题跟随当前排序/分类。请求期间显示加载反馈，不允许操作旧筛选结果；失败只显示一次错误说明，提供重试和本地入口。旧响应不得覆盖新请求的内容与加载状态。验收场景见[浏览精修计划](../../plans/2026-09-08-browse-refinement.md)。

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

#### Scenario: 桌面只读详情与重试

- GIVEN 用户点击广场卡片
- WHEN 详情读取成功或失败
- THEN 展示完整正文，或展示可重试错误，且允许关闭
- AND 仅浏览不写入本地库、不上报下载统计；关闭或切换条目后旧响应不得重新打开或覆盖当前详情
- AND 下载进行中禁用重复点击，成功显示反馈，失败保留详情供重试

#### Scenario: 浏览排序与模型筛选

- GIVEN 预发种子含不同标题与模型
- WHEN 分别请求 `sort=recommended`、`latest`、`hot`
- THEN 三种顺序可区分
- AND `hot` 按匿名下载次数降序，次数相同则按标题升序
- AND `model` 查询只返回该模型
- AND 不得声称这是生产热度算法

#### Scenario: 已登录收藏排序

- GIVEN 用户已登录并收藏了某条
- WHEN 请求 `sort=favorites` 且携带 Access
- THEN 列表含该条
- AND 未登录时收藏排序为空列表

### Requirement: 内容可见性与访问开关

offline/trashed 条目 MUST 从所有公开读取路径排除，已知 ID 详情和正文为 404，下载计数不增长。下架不删除本地下载副本。关闭匿名浏览后已登录用户仍可读取，匿名列表为空、详情/正文为 401；数据库或配置失败不得默认放行。

#### Scenario: 下架后已知 ID

- GIVEN 管理人员已下架广场条目
- WHEN 匿名或已登录用户访问其详情、正文、收藏列表或上报下载
- THEN 不返回该条目或增加计数；恢复上线后保持同一 ID

#### Scenario: 全部下架后重启

- GIVEN 库内存在内容但全部下架
- WHEN 服务重启
- THEN 不重新植入演示内容或清空原有记录，仍能在管理端恢复

### Requirement: 下载与收藏分离（M5）

「下载」MUST 把远端记录复制到本地 SQLite，不要求登录。「收藏」MUST 是账号关系，未登录时打开登录并说明原因。

桌面下载 MUST 根据未删除本地提示词的 `remote_id` 判断已下载，卡片与详情统一显示状态；重复或并发入口不得创建重复副本。成功/失败使用固定可见轻提示，成功刷新本地数量；删除同来源全部副本后允许重新下载。场景见[下载反馈计划](../../plans/2026-09-08-download-feedback.md)。

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

### Requirement: 匿名下载统计

开启「匿名下载统计」且本地下载成功后，客户端 MUST `POST /v1/square/items/{id}/downloads`，MUST NOT 带 Authorization，MUST NOT 发送账号、正文或标题。关闭时 MUST NOT 请求。统计失败 MUST NOT 阻断下载。服务端 MUST 接受匿名 POST，未知 id MUST 404，MUST NOT 把 GET 正文当成一次统计，MUST NOT 记录谁下载。

#### Scenario: 打开后上报条目 id

- GIVEN 开关已打开且下载成功
- WHEN 客户端上报
- THEN 该 id 计数加一
- AND 请求无 Authorization

#### Scenario: 关闭不静默上报

- GIVEN 开关关闭
- WHEN 用户下载
- THEN 服务端计数不变

#### Scenario: GET 正文不加次数

- GIVEN 某条计数为零
- WHEN 匿名 GET `/content`
- THEN 计数仍为零
- AND 热门顺序不因此改变

### Requirement: 自动同步收藏队列

开启「自动同步收藏与发布草稿」且已登录时，收藏或取消收藏失败 MUST 写入本机队列，MUST NOT 因此新增本地 `source=downloaded` 副本，MUST NOT 假装已经到达服务器。立即同步 MUST 把本账号队列送出。开关关闭时失败 MUST 不入队。同一收藏 id 后写 MUST 覆盖先写。

#### Scenario: 断网收藏入队

- GIVEN 用户已登录且开关已打开，收藏请求失败
- WHEN 用户收藏一条广场条目
- THEN 本机队列含该收藏
- AND 本地库不新增副本

#### Scenario: 关闭开关不入队

- GIVEN 用户已登录且开关关闭
- WHEN 收藏请求失败
- THEN 队列仍为空

#### Scenario: 冲刷后到达服务端

- GIVEN 队列中有一条收藏且网络已恢复
- WHEN 立即同步
- THEN 账号收藏关系被写入
- AND 队列清空

#### Scenario: 同一收藏后写覆盖

- GIVEN 开关已打开且同一 id 先入队收藏再入队取消
- WHEN 查看队列
- THEN 只保留取消收藏

#### Scenario: 新请求成功后不重放旧操作

- GIVEN 同一条目的旧收藏操作仍在离线队列
- WHEN 用户的新取消收藏请求成功
- THEN 清除该条目的旧队列操作，不得再把它收藏回来
- AND 并发队列操作顺序执行，切换账号后不使用新账号发送旧账号任务

## 测试映射

### Requirement: 合集下载

合集详情 MUST 显示成员标题与正文。桌面下载 MUST 在同一事务中新增合集及全部成员副本，保留各自分类/模型；每个成员标记 downloaded，MUST NOT 将合集转换为空白提示词。桌面本地仍有该合集的下载成员时视为已下载，不重复导入整套。历史缺成员快照的合集 MUST 提示不可下载。Web 下载 MUST 同样保留合集及成员归属，并能打开成员。

#### Scenario: 合集完整副本

- GIVEN 广场合集有两条成员
- WHEN 匿名下载
- THEN 本地新增一个合集和两条带归属的提示词，源内容不变
- AND 下载失败不留下空合集或部分成员，不上报成功统计

### Requirement: 分类端到端

分类树 MUST 显示全广场 online 条目数量，大分类包含子分类，全部包含未分类；数量不受当前搜索、模型或收藏筛选影响。首批请求用数据库聚合返回，续页不重复统计，加载失败不显示假零。本地计数保持独立。场景见[分类数量计划](../../plans/2026-09-09-category-counts.md)。

广场条目与正文 MUST 返回 `category_id`、`model`。`category_id` 查询 MUST 支持系统大分类（含子类）或小分类精确过滤。下载 MUST 保留这些字段；历史缺字段仍作为未分类可下载。收藏视图 MUST 同时遵守分类、搜索及模型筛选。

#### Scenario: 分类下载

- GIVEN 一条图片生成人像分类、Flux 模型的已审核提示词
- WHEN 用户按图片大分类浏览并下载
- THEN 只显示对应分类内容，下载后仍属于人像且模型为 Flux
- AND 软件开发筛选不出现该条

回归：后端分类发布审核下载测试、`square.test.js` 和 `WorkbenchShell.spec.js`。

| 场景 | 测试 |
|---|---|
| 已清空实例重启 | `cleared_square_can_persistently_disable_demo_seeding`；本机重启空列表/旧正文 404 验收见清空计划 |
| M2 构建无广场请求 | 已由 M5 浏览替代；离线不阻断本地 |
| 离线 | `WorkbenchShell.spec.js` shows a non-blocking offline notice and can return to local；`LauncherApp.spec.js` does not request square while searching locally |
| 未登录下载 | `WorkbenchShell.spec.js` downloads a square prompt without login as source=downloaded；`square.test.js` writes a local copy with source=downloaded；`imports_downloaded_prompt_with_source`；`serves_square_item_content_without_login` |
| 打开后上报条目 id | `square.test.js` posts anonymous download stats without auth after a successful download when the setting is on；`backend` `record_anonymous_download_increments_count_without_auth` |
| 关闭不静默上报 | `square.test.js` does not post download stats when the setting is off；`WorkbenchShell.spec.js` does not record anonymous download stats when the setting is off |
| GET 正文不加次数 | `backend` `get_content_does_not_count_as_anonymous_stats`；`backend` `missing_item_download_stat_is_404` |
| 未登录收藏 | `WorkbenchShell.spec.js` opens login from favorite without writing a local copy |
| 已登录收藏 | `WorkbenchShell.spec.js` favorites a square item while logged in without writing a local copy；`square.test.js` puts a favorite without writing a local copy；`backend` `put_favorite_lists_for_account` |
| 取消收藏 | `WorkbenchShell.spec.js` keeps a downloaded copy after unfavorite；`backend` `delete_favorite_removes_account_relation` |
| 断网收藏入队 | `syncQueue.test.js` queues a favorite when auto-sync is on and the request fails；`WorkbenchShell.spec.js` queues a favorite while offline when auto-sync is on and flushes on sync now |
| 关闭开关不入队 | `syncQueue.test.js` does not queue a favorite when auto-sync is off |
| 冲刷后到达服务端 | `syncQueue.test.js` flushes a queued favorite when the transport recovers；`WorkbenchShell.spec.js` queues a favorite while offline when auto-sync is on and flushes on sync now |
| 同一收藏后写覆盖 | `syncQueue.test.js` keeps the later favorite write for the same id |
| 合同 path 与匿名下载 | `squareContract.test.js` lists every contract path |
| 浏览混排 | `WorkbenchShell.spec.js` shows square items in the content grid not the category tree；`backend` `lists_square_items_without_login` |
| 条目详情 | `backend` `serves_square_item_without_login` |
| 浏览排序与模型筛选 | `backend` `sorts_recommended_latest_and_hot_apart`；`backend` `record_anonymous_download_increments_count_without_auth`；`backend` `anonymous_download_count_survives_new_appstate_on_postgres`；`square.test.js` forwards the selected model to the square transport；`WorkbenchShell.spec.js` filters square items by the selected model |
| 已登录收藏排序 | `backend` `favorites_sort_requires_login` |
| 进程重启后列表仍在 | `backend` `publication_favorite_and_settings_survive_postgres` |
