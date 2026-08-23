# 如何在本机工作

M0–M4 已关闭。当前按 M5 做广场浏览、匿名下载与登录。浏览器可预览工作台；桌面窗口才能写 SQLite、登记全局快捷键、备份库文件、持久化 Refresh。

已验证平台：**macOS**。不要把 Windows / Linux 写成已支持。

不要同时跑 `npm run dev` 和 `npm run tauri dev`：两者都占用 `1420`。

## 检查文档

```bash
./scripts/docs-check
```

必须退出码 0。

## 自动化测试

```bash
cd desktop
npm install
npm test
cd src-tauri
unset CARGO_TARGET_DIR
cargo test --locked
cd ../../backend
unset CARGO_TARGET_DIR
cargo test --locked
```

M5 会话服务（可选，浏览器预览不持久化 Refresh）：

```bash
cd backend
unset CARGO_TARGET_DIR
cargo run
```

默认 `127.0.0.1:8787`。桌面窗口登录会把 Refresh 写入系统钥匙串。

## 浏览器验证（最快）

```bash
cd desktop
npm install
npm run dev
```

打开终端里提示的地址（默认 `http://localhost:1420`）。

请确认：

1. 顶栏、左侧栏、内容区、底栏都在，默认是「本地提示词」。
2. 可新建提示词；底栏显示「SQLite 未接入」（浏览器走内存库，预期）。
3. 点「提示词广场」：API 未开时出现「当前离线」和「前往本地」；本地库仍可用。启动器只搜本地。API 开着时，点卡片「下载」不弹登录，切回本地能看到该条；点「收藏」登录原因含「收藏」，本地不因此多出一条。
4. 点顶栏「搜索」打开独立启动器页（新窗口或 `/launcher.html`）。浏览器启动器有自己的空内存库。
5. 设置 → 数据与备份：JSON 可预览再导入；点「备份库文件」应提示「仅桌面窗口支持库文件备份」。

## 桌面窗口验证

需要本机已装 Rust（`cargo`）和 Node。

```bash
cd desktop
npm install
npm run tauri dev
```

请确认：

1. 出现「提示方舟」主窗口，底栏为「SQLite 就绪」。
2. 新建一条提示词后，设置 → 数据与备份 →「备份库文件」，记下路径。
3. 再新建另一条，用上一步路径「恢复库文件」，列表回到备份时的内容。
4. 点顶栏「搜索」出现独立启动器窗口（label `launcher`），不是主窗口遮罩。
5. 默认 `Control+Space` 唤起启动器；与系统冲突时设置页报错且不保存。

第一次编译 Tauri 可能要几分钟。

启动器 1 万条查询预算（本机 `release`，不作为 CI 红灯）：

```bash
./scripts/launcher-search-bench
```

发行前手工表见 [发行前 QA](release-qa.md)。当前没有商店包，也没有可公开下载声明。

## 阅读顺序（新人）

1. [README](../../README.md) 与 [文档体系](../README.md)
2. [宪法](../constitution.md)
3. [PRD](../product/prd.md)
4. [路线图](../product/roadmap.md)
5. 按任务打开一份 `docs/specs/<能力>/spec.md`
