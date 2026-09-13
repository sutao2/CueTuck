# Skill 广场来源扩充

| 字段 | 值 |
|---|---|
| 状态 | 已实现；下载抽测的限流边界见下文 |
| 授权 | 用户要求「广场的 skill 太少了，多加一些」 |
| 规格 | [Skills 规格](../specs/skills/spec.md) |

## 实施前方案

扩充内置公开仓库，覆盖开发、AI、科研、营销、设计与安全。沿用当前来源按需加载、固定提交预览与完整包安装；不自动安装 Skill。来源选项显示用途和仓库名称，支持关键词查找。仅纳入真实可访问、有标准 SKILL.md 目录的仓库；数量取实时目录结果，不硬编码热度。

## 验收步骤

- [x] 核查候选仓库可访问性、目录、许可及条目数量。
- [x] 扩充默认来源并保留自定义来源，验证来源搜索与切换。
- [x] 已核实所有新增来源目录；原生下载器完成两个来源样例，其余抽测受匿名接口限流中断，见下文。
- [x] 14 项组件测试、Vite 与 Tauri debug 构建、文档检查通过；随代码提交。

## 来源核查（2026-09-13）

以下是 GitHub 实际目录结果；各仓库条目未跨来源去重，数量会随上游更新变化。许可是仓库声明，具体包仍显示自己的许可。新增 11 个来源，共 621 个目录条目；原有 4 个来源保留。Next Skills 候选未发现标准 SKILL.md，未纳入。

| 来源 | 条目 | 仓库许可 | 核查提交 |
|---|---:|---|---|
| [huggingface/skills](https://github.com/huggingface/skills) | 26 | Apache-2.0 | `f3186efbbc322121eb5d0f31e8a1d669ee961159` |
| [MicrosoftDocs/Agent-Skills](https://github.com/MicrosoftDocs/Agent-Skills) | 202 | CC-BY-4.0 | `90ec55f3dc95837df8b9f7bd6265fb5935ccdca6` |
| [cloudflare/skills](https://github.com/cloudflare/skills) | 14 | Apache-2.0 | `b052c32bab7dd493513260228a36c88294f343f1` |
| [trailofbits/skills](https://github.com/trailofbits/skills) | 83 | CC-BY-SA-4.0 | `321ccfe628eca0d314b0ee4eaffcdd8a05639aaf` |
| [supabase/agent-skills](https://github.com/supabase/agent-skills) | 2 | MIT | `8331f910845103c08d51f6ca1d86ebb7d1f745e3` |
| [getsentry/skills](https://github.com/getsentry/skills) | 27 | Apache-2.0 | `c2f99a5b04b4cd992ec3022d7c2c3e23e938d241` |
| [remotion-dev/skills](https://github.com/remotion-dev/skills) | 12 | 未知 | `bd566b65d521b40fe92e1f26766e82de9e291693` |
| [expo/skills](https://github.com/expo/skills) | 26 | MIT | `be95528712e3709b786d573edae90d14c67949a8` |
| [obra/superpowers](https://github.com/obra/superpowers) | 14 | MIT | `b36e0829c6d0140e93cfef2ca599b1b07d4a7797` |
| [coreyhaines31/marketingskills](https://github.com/coreyhaines31/marketingskills) | 50 | MIT | `5b2c0007766c6a1cf1d53fd8fc73e979e0821022` |
| [K-Dense-AI/claude-scientific-skills](https://github.com/K-Dense-AI/claude-scientific-skills) | 165 | MIT | `fa66f483619e36827a9521302666067821e052c9` |

## 验收结果

- 组件 14 项通过，包括用途搜索、选择新增来源后的真实请求参数、自定义来源保留及大小写去重；切换来源不触发安装。
- Vite 生产构建与独立 `CueTuck Skills QA.app` debug 构建通过。已重启测试客户端，原生界面显示「公开来源 15 个」。正式发行包未更新。
- 使用 GitHub API 核实全部 11 个新增仓库目录，无截断；记录见上表与 `/tmp/cuetuck-skill-source-catalogs.jsonl`。
- 原生完整包下载抽测：Hugging Face `hf-mcp/skills/hf-mcp`（1 文件，4968 字节）、Microsoft `skills/azure-active-directory-b2c`（1 文件，43747 字节）均通过包摘要复核。继续到 Cloudflare 时匿名 GitHub API 限流，整组网络测试因此失败；其余 9 个来源未完成原生下载抽测，不宣称全部包已测试。保留显式忽略的网络测试以便限流恢复后复验，不添加认证或绕过限流。
- 所有下载仅写系统临时目录；没有安装或执行任何第三方 Skill。
- 日志：`/tmp/cuetuck-expanded-sources-ui.log`、`/tmp/cuetuck-expanded-sources-native.log`、`/tmp/cuetuck-expanded-sources-build.log`、`/tmp/cuetuck-expanded-sources-app.log`。
