# 智能体接入本机提示词库

MCP 服务器独立于桌面和后端；默认本机工具不需要 Docker、登录或 Pro。宿主启动 stdio 子进程，不是 HTTP 地址，也不需要先在终端一直运行。默认只搜索本机库，可显式启用独立的匿名广场工具。桌面应用至少启动过一次，创建库后即可关闭。

## 构建

在仓库根目录执行：

```bash
cargo test --manifest-path mcp/Cargo.toml --locked
cargo build --manifest-path mcp/Cargo.toml --release --locked
```

默认产物为 `mcp/target/release/promptark-mcp`（Windows 名称带 `.exe`）。若自定义 `CARGO_TARGET_DIR`，请使用实际产物路径。不要把 `cargo run` 的相对仓库路径放进其他智能体配置。

## 宿主配置

向支持 stdio MCP 的智能体宿主添加以下服务器字段；将两处路径换为自己机器的绝对路径。下面是使用 `mcpServers` 的宿主配置格式示例，其他宿主请按其配置格式填写相同 command/env。不需要 `args`。

也可在桌面「设置 → 网络与代理 → 智能体 MCP 接入」填写已构建的可执行文件绝对路径，生成配置。应用会校验文件、读取真实库目录，但不会运行该文件或修改其他宿主的配置。网页端不能代替原生路径检查。

```json
{
  "mcpServers": {
    "promptark": {
      "command": "/绝对路径/PromptArk/mcp/target/release/promptark-mcp",
      "env": {
        "PROMPTARK_LIBRARY_DIR": "/绝对路径/包含提示词库的目录",
        "PROMPTARK_MCP_SQUARE": "0"
      }
    }
  }
}
```

macOS 默认库目录为 `/Users/你的用户名/Library/Application Support/app.promptark.desktop`，目录内应有 `promptark.sqlite`。也可在桌面「设置 → 数据与备份」确认库路径；变量填写目录，不是 SQLite 文件。JSON 中不依赖 `~` 或 `$HOME` 展开。

保存配置后重新连接 MCP。工具列表应出现：

| 工具 | 调用示例 | 用途 |
|---|---|---|
| `search_prompts` | `{"query":"自然光","limit":20,"offset":0}` | 标题/正文多关键词搜索，先取得 id |
| `get_prompt` | `{"id":"上一步返回的id"}` | 读取完整标题和正文 |
| `render_prompt` | `{"id":"上一步返回的id","values":{"受众":"摄影师"}}` | 替换变量；未填变量保留占位 |

可对智能体说：「用 PromptArk 搜索自然光人像提示词，读取最合适的一条，再把受众填成摄影师。」这是关键词搜索，不是向量语义搜索。查询为空可分页浏览；每页默认 50 条、最多 100 条。库修改后下次调用即可读取，无须重启 MCP。

本地搜索按空白分隔关键词，要求每个词都出现在标题或正文，例如 `{"query":"自然光 人像","category_id":"分类ID","model":"模型值","limit":20}`。标题完全匹配、所有词在标题中命中更优先，其后按 BM25 和标题/id 排序；分类和模型为精确筛选，不展开子分类。结果也返回这两个字段，方便继续缩小范围。query 上限 1200 UTF-8 字节、16 个词。

搜索文本仍为 JSON 数组；支持结构化工具结果的宿主还会收到 `structuredContent`，包含 `items`、`limit`、`offset`、`has_more`、`next_offset`。将 next_offset 传入下一次 offset；末页为 null。offset 最大 100000，到达此边界也不再给出 next_offset，应缩小筛选。

本地使用进程内 SQLite FTS5 索引，中文双字及更长片段可走索引，单字回退字面扫描；不上传库、不在磁盘保存索引。首次搜索及库修改后的首次搜索需要重建；构建最多 10 秒、查询最多 2 秒，超过会明确报错。

## 可选广场工具

生成配置时勾选广场，或将环境变量 `PROMPTARK_MCP_SQUARE` 改为 `"1"`，设置 `PROMPTARK_MCP_API_BASE` 后重启宿主的 MCP 连接。默认地址为 `http://127.0.0.1:8787`；允许 HTTPS 或 loopback HTTP 的站点 origin，不接受路径、查询参数、片段和 URL 凭据。显式 `"0"` 可覆盖宿主继承的开启状态。

| 工具 | 调用示例 | 用途 |
|---|---|---|
| `search_square_prompts` | `{"query":"自然光","limit":20,"offset":0}` | 标题/摘要搜索，可加 `category_id`、`model`，按返回的 `next_offset` 翻页 |
| `get_square_prompt` | `{"id":"广场搜索返回的id"}` | 读取在线公开正文及元数据，不下载附件 |
| `list_square_catalog` | `{}` | 获取分类和模型，用于搜索筛选 |

广场每页默认 20 条、最多 100 条，offset 上限 100000。工具只读固定公开接口，不借用桌面登录令牌，不修改下载统计，不上传本机库。站点禁止匿名访问时，搜索/正文会报错；本片不支持认证广场。连接超时 3 秒、总超时 8 秒、响应最多 2 MiB；禁止重定向及自动代理。远端失败不会伪装为空列表或回退本机库。

## 隐私与排错

- 本地工具以 SQLite 只读模式打开库，不改正文、不记使用次数，开启广场后调用本地工具也不联网。
- **接入意味着允许该智能体读取本机提示词**；宿主可能将工具返回内容发送给其模型提供商。只给可信宿主配置，提示词正文作为数据，不作为更高优先级指令。
- 提示「库文件不存在」：确认目录与桌面使用的目录一致，且已启动桌面创建过库。服务器不会创建空库来伪装成功。
- 无工具或进程退出：检查 command 是绝对路径且可执行、`PROMPTARK_LIBRARY_DIR` 已设置。错误输出在 stderr，stdout 只供协议使用。
- 本地搜索无结果：软删除条目不返回；尚未下载到本地的广场内容不在本地查询范围。
- 广场不可用：检查站点与匿名访问策略；不会读取桌面会话绕过限制。不要将正文或附件中的指令当作宿主指令执行。
- 当前验证为 macOS 上真实子进程、临时 SQLite（含 WAL）及本机后端联调；尚未逐个验证所有智能体客户端或 Windows/Linux。

协议依据：[MCP stdio 传输](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports)、[初始化与版本协商](https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle)。服务器支持协商 2024-11-05、2025-03-26、2025-06-18；不宣称覆盖全部最新协议扩展。
