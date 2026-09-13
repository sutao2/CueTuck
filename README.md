# CueTuck · 唤词

Your prompts, a shortcut away.

本地优先的桌面提示词工作台。主窗口按「提示词软件 2」原型重建；全局启动器沿用旧 Prompt Launcher 的独立窗口、快捷键与粘贴链路。

M0–M9 已关闭。本机工作台、独立启动器、库文件备份、设置十类、本仓库广场 API、独立管理台、已登录收藏、令牌轮换、管理员身份、个人库立即同步、Google / GitHub 登录、自动更新检查与安装、预发账单可用。浏览器工作台与本机 MCP 独立交付，不进桌面包。已验证平台是 macOS，未验证 Windows / Linux。无商店包、无公开下载声明。商店上架与生产托管不得假装接通。

## 先读哪份

| 读者 | 打开 |
|---|---|
| 人 | [docs/README.md](docs/README.md) 理解体系，[docs/INDEX.md](docs/INDEX.md) 查文件 |
| 参与 | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Agent | [AGENTS.md](AGENTS.md)，再按 INDEX 打开 1～2 个文件 |
| 产品范围 | [docs/product/prd.md](docs/product/prd.md) |
| 非协商原则 | [docs/constitution.md](docs/constitution.md) |
| 测试门禁 | [docs/reference/test-gates.md](docs/reference/test-gates.md) |

## 状态

- 产品名：CueTuck · 唤词
- 阶段：M9 已关闭；不能诚实做完的项见 [deferred.md](docs/plans/deferred.md)
- 版本：未发行；无商店包、无公开下载声明
- 旧仓库（只读参考）：`../PromptLauncher`

## 本机验证

见 [如何在本机工作](docs/how-to/local-dev.md)。最短路径：

```bash
cd desktop
npm install
npm test
npm run dev
```

桌面窗口：`npm run tauri dev`。

## 文档检查

```bash
./scripts/docs-check
```
