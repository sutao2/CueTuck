# macOS Window Chrome Implementation Plan

> **For agentic workers:** 按任务顺序执行。不要接广场或开 M5。

**Goal:** 主窗口与启动器在 macOS 上用系统红绿灯 / overlay 与 `⌃Space`，不再像 Windows 无框工具窗。

**Architecture:** 纯函数判定宿主并格式化快捷键。主窗口 Tauri `titleBarStyle: Overlay`。启动器保持独立无框窗口，做成圆角面板。Windows 宿主仍可用 `Ctrl Space`，不宣传为已验证。

**Tech Stack:** 既有 Vue、Vitest、Tauri 2。

## Global Constraints

- 启动器 label 仍是 `launcher`，不是覆盖层。
- 原型只定四区节奏，不定 Windows 风窗框。
- 没有本计划之外的应用文件。
- 每任务结束跑 `./scripts/docs-check`。

---

### Task 1: 宿主窗框纯函数

**Files:** `desktop/src/platform/windowChrome.js`、`windowChrome.test.js`

- [ ] 失败测试：`macos` → inset 78、`⌃Space`；`windows` → inset 0、`Ctrl Space`
- [ ] 最小实现 `detectHost` / `trafficLightInsetPx` / `formatShortcutLabel` / `applyHostChrome`
- [ ] `cd desktop && npm test -- src/platform/windowChrome.test.js`

### Task 2: 主窗口顶栏

**Files:** `WorkbenchShell.vue`、`WorkbenchShell.spec.js`、`workbench-chrome.css`、`tokens.css`、`tauri.conf.json`

- [ ] 失败测试：`host=macos` 时 titlebar 有 `host-mac`，kbd 为 `⌃Space`，无右侧窗控
- [ ] 顶栏给红绿灯 inset，隐藏左侧 P 标；主窗口 `titleBarStyle: Overlay` + `hiddenTitle`
- [ ] `cd desktop && npm test`

### Task 3: 启动器面板

**Files:** `LauncherApp.vue`、`LauncherApp.spec.js`、`launcher.html` / `tokens.css`、`tauri.conf.json`

- [ ] 失败测试：`host=macos` 时根节点有 `host-mac`
- [ ] 圆角面板 + 透明窗；仍是独立 `launcher`
- [ ] 规格映射改为测试名；更新 status

- [ ] `./scripts/docs-check`
