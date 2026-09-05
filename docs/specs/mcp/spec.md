# 本机 MCP

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M9 实现 |
| 关联 | [本地提示词](../library/spec.md) · [变量](../variables/spec.md) · [ADR 0011](../../architecture/decisions/0011-web-and-mcp.md) |

## Purpose

让 Codex、Claude、Cursor 等 MCP 宿主查询本机提示词库。只走本机 SQLite，不搜广场。

## Requirements

### Requirement: stdio MCP

系统 MUST 提供 stdio MCP 服务器。宿主用标准 MCP 初始化后 MUST 能列出并调用工具。MUST NOT 以 ChatGPT 插件或单一 IDE 扩展作为第一交付形态。

#### Scenario: 列出工具

- GIVEN MCP 进程已启动且完成 initialize
- WHEN 宿主请求工具列表
- THEN 包含 `search_prompts`、`get_prompt`、`render_prompt`

### Requirement: 只读本机库

查询 MUST 针对 `PROMPTARK_LIBRARY_DIR` 下的 `promptark.sqlite`。MUST 忽略已软删条目。库文件不存在时 MUST 返回明确错误，MUST NOT 编造提示词。MUST NOT 请求广场或管理 HTTP。

连接 MUST 使用 SQLite 只读打开模式；未配置目录时进程 MUST 在 stderr 给出配置提示并退出，不猜测当前工作目录。搜索默认最多 50 条，允许 `limit`（1–100）和非负整数 `offset` 分页，按标题/id 稳定排序；`%`、`_` 作为普通查询字符。参数类型错误必须报错，不得降级为全库搜索。

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

### Requirement: 不请求广场

MCP 进程在搜索与读取时 MUST NOT 发起广场或管理接口请求。

#### Scenario: 搜索不联网

- GIVEN MCP 已启动
- WHEN 调用 `search_prompts`
- THEN 不出现对广场或 `/v1/admin` 的 HTTP 请求

## 测试映射

| 场景 | 测试 |
|---|---|
| 有界搜索且不写库 | `connection_is_read_only_and_search_is_bounded_and_literal`、`rejects_bad_arguments_without_searching_all_prompts` |
| 宿主保持连接 | `mcp/tests/stdio.rs` 真实进程初始化/查询/读取/渲染/解析错误/软删除/缺库验证 |
| 列出工具 | `mcp` `lists_required_tools` |
| 按标题命中 | `mcp` `search_hits_title` |
| 缺库文件 | `mcp` `search_missing_library_errors` |
| 未填保留占位 | `mcp` `render_keeps_unfilled_placeholder` |
| 搜索不联网 | `mcp` `search_has_no_http_client` |
