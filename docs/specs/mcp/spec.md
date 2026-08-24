# 本机 MCP

| 字段 | 值 |
|---|---|
| 状态 | 已指定，M9 目标 |
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
| 列出工具 | 未开始 |
| 按标题命中 | 未开始 |
| 缺库文件 | 未开始 |
| 未填保留占位 | 未开始 |
| 搜索不联网 | 未开始 |
