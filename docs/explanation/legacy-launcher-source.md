# 说明：启动器代码源

启动器为什么不跟原型走。

原型的快捷搜索是 DOM 覆盖层，适合演示，不处理真实焦点、托盘和系统粘贴。旧产品把启动器做成独立窗口，并在 Rust 里处理唤起保护期、隐藏后交还焦点、模拟粘贴、macOS 辅助功能读选中文本。这些是桌面约束，不是皮肤。

移植范围（旧仓库相对路径）：

- `prompt-launcher/src/launcher.js`、`LauncherApp.vue`、`composables/useLauncher.js`
- `prompt-launcher/src/api/launcher*.js`、`utils/variables.js` 中与启动器共用的解析
- `prompt-launcher/src-tauri/src/commands/launcher.rs`、`paste.rs`
- 全局快捷键与窗口权限配置中仅启动器需要的部分

不要整仓库复制旧前端。主窗口按原型重写；启动器接新 SQLite，而不是旧 `repository/index.js` 全量搬家。
