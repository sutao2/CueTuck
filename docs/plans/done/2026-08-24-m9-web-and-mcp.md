# 完成记录：M9 浏览器工作台与 MCP

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-24 |
| 里程碑 | M9 |
| 计划 | [2026-08-24-m9-web-and-mcp.md](../2026-08-24-m9-web-and-mcp.md) |

## 退出标准

- [x] web 规格场景有测试映射
- [x] mcp 规格场景有测试映射
- [x] 桌面包仍不含 `web/` 与 `mcp/` 应用入口
- [x] MCP 搜索不请求广场
- [x] done 记录 + status 更新

## 命令与结果

```text
cd web && npm test
Test Files  1 passed (1)
Tests  4 passed (4)

cd desktop && npx vitest run src/platform/packageIsolation.test.js
Test Files  1 passed (1)
Tests  3 passed (3)

cd mcp && unset CARGO_TARGET_DIR && cargo test --locked
5 passed; 0 failed; 0 ignored

./scripts/docs-check
docs-check 通过（124 个 Markdown 文件）。
```

独立 `web/` 流体壳：可收起侧栏、标签页内存库新建/列出/打开，不进桌面包，不声称与桌面 SQLite 已同步。独立 `mcp/` stdio：`search_prompts` / `get_prompt` / `render_prompt` 读本机库，缺文件报错，未填保留 `{{名称}}`，无 HTTP 客户端。启动器仍只搜本地。未做原生移动端、ChatGPT 插件、云同步。Windows NSIS 工作流未因本机交叉编译跑通，GitHub Actions 曾因账号额度失败；不声称 Windows 已验证。

## 文档

- 更新的规格：web、mcp
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 浏览器工作台无使用向导、分类、合集；广场在 web 里仍是空说明
- 密码 KDF、Postgres、OAuth、账单、商店上架
- 广场预发：排序/筛选弱；审核通过不进广场列表；`GET /v1/square/items/{id}` 未路由
- Windows / Linux 开机启动与托盘尚未验证
- 云同步引擎与自动更新安装须另立项
