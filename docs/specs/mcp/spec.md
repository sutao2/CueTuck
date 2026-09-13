# 本机 MCP

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M9 实现 |
| 关联 | [本地提示词](../library/spec.md) · [变量](../variables/spec.md) · [ADR 0021](../../architecture/decisions/0021-mcp-square-opt-in.md) |

## Purpose

让支持 stdio 的 MCP 宿主查询本机提示词库；按 [ADR 0021](../../architecture/decisions/0021-mcp-square-opt-in.md) 显式启用独立广场工具。

## Requirements

### Requirement: stdio MCP

系统 MUST 提供 stdio MCP 服务器。宿主用标准 MCP 初始化后 MUST 能列出并调用工具。MUST NOT 以 ChatGPT 插件或单一 IDE 扩展作为第一交付形态。

#### Scenario: 列出工具

- GIVEN MCP 进程已启动且完成 initialize
- WHEN 宿主请求工具列表
- THEN 包含 `search_prompts`、`get_prompt`、`render_prompt`

### Requirement: 只读本机库

本地工具 MUST 针对 `PROMPTARK_LIBRARY_DIR` 下的 `promptark.sqlite`。MUST 忽略已软删条目。库文件不存在时 MUST 返回明确错误，MUST NOT 编造提示词。本地工具 MUST NOT 请求广场或管理 HTTP。

连接 MUST 使用 SQLite 只读打开模式；未配置目录时进程 MUST 在 stderr 给出配置提示并退出，不猜测当前工作目录。搜索默认最多 50 条，允许 `limit`（1–100）和 `offset`（0–100000）分页，空查询按标题/id 稳定排序；`%`、`_` 作为普通查询字符。参数类型错误必须报错，不得降级为全库搜索。

搜索 MUST 支持空白分隔的多关键词（全部命中标题或正文）、不区分大小写的字面匹配、标题优先相关性排序及 `category_id` / `model` 精确筛选。query 最多 1200 UTF-8 字节、16 个词，筛选值最多 200 字节。返回的文本保持 JSON 数组兼容；`structuredContent` 提供 items、limit、offset、has_more、next_offset。MUST NOT 将关键词作为 SQL 或 FTS 语法。

搜索索引仅存在于进程内存，源库只读；源库提交、软删除或替换后下一次查询 MUST 刷新，库消失 MUST 报错。首次/刷新索引最多 10 秒、查询最多 2 秒（SQLite 进度回调及源库锁等待有界）。双字符及以上使用 FTS5 片段索引，单字符允许扫描；不承诺分词、同义词或向量语义能力。

#### Scenario: 多词排序与筛选分页

- GIVEN 中文/英文多个词分别出现在标题和正文，存在同名与不同分类/模型条目
- WHEN 多词搜索并指定筛选与分页
- THEN 所有词字面命中，标题优先且稳定，末页 next_offset 为 null
- AND 旧文本数组可读取，structuredContent 含完整分页状态

#### Scenario: 索引刷新

- GIVEN MCP 已建立内存索引
- WHEN 外部提交 WAL 更新、软删除或替换库
- THEN 下次搜索反映最新已提交内容；库缺失不返回缓存
- AND 不修改本机 SQLite 内容

#### Scenario: 有界搜索且不写库

- GIVEN 含超过一页数据及软删除条目的本机库
- WHEN 智能体分页搜索、读取或填变量
- THEN 每页不超过 limit，忽略软删除，数据库内容不变
- AND 缺文件不创建空库，非法参数返回错误

#### Scenario: 宿主保持连接

- GIVEN 宿主已初始化 stdio 进程
- WHEN 发送 ping、通知、非法 JSON 再发送正常搜索
- THEN ping 返回空结果，通知不响应，非法 JSON 返回解析错误且进程继续处理正常请求

#### Scenario: 按标题命中

- GIVEN 库中有标题为「自然光群像」且未删除的提示词
- WHEN 调用 `search_prompts` 且查询含「自然光」
- THEN 结果含该条 id 与标题

#### Scenario: 缺库文件

- GIVEN `PROMPTARK_LIBRARY_DIR` 下没有 `promptark.sqlite`
- WHEN 调用 `search_prompts`
- THEN 返回错误且不含假条目

### Requirement: 读取与渲染

`get_prompt` MUST 返回标题与正文。`render_prompt` MUST 使用与桌面相同的 `{{名称}}` 规则；未填 MUST 保留 `{{名称}}`。

#### Scenario: 未填保留占位

- GIVEN 正文为 `给 {{受众}} 的说明`
- WHEN 调用 `render_prompt` 且不提供受众
- THEN 结果仍包含 `{{受众}}`

### Requirement: 本地工具不请求广场

本地工具搜索与读取 MUST NOT 发起广场或管理接口请求。广场工具、启动授权、限制与配置生成 MUST 满足 [P2 计划](../../plans/2026-09-08-mcp-square.md) 的场景；默认不列出广场工具，不能由工具参数开启网络。

#### Scenario: 搜索不联网

- GIVEN MCP 已启动
- WHEN 调用 `search_prompts`
- THEN 不出现对广场或 `/v1/admin` 的 HTTP 请求

## 测试映射

| 场景 | 测试 |
|---|---|
| 多词排序与筛选分页、索引刷新 | `mcp/tests/search.rs` 中文/字面/分页/WAL/替换/参数/取消测试 |
| 有界搜索且不写库 | `connection_is_read_only_and_search_is_bounded_and_literal`、`rejects_bad_arguments_without_searching_all_prompts` |
| 宿主保持连接 | `mcp/tests/stdio.rs` 真实进程初始化/查询/读取/渲染/解析错误/软删除/缺库验证 |
| 列出工具 | `mcp` `lists_required_tools` |
| 按标题命中 | `mcp` `search_hits_title` |
| 缺库文件 | `mcp` `search_missing_library_errors` |
| 未填保留占位 | `mcp` `render_keeps_unfilled_placeholder` |
| 搜索不联网 | `mcp` `local_default_rejects_remote_calls`；`mcp/tests/square.rs` 默认与启用后本地工具均无 HTTP |
