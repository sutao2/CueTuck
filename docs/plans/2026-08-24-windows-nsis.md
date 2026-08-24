# Windows NSIS 当前功能包

> **给 Agent：** 不是新里程碑。只让当前桌面功能能打出 Windows NSIS。不要声称 Windows 已验证。不要上架商店。不要假装云同步 / OAuth / 自动更新已接通。管理端不进桌面包。

**Goal:** Windows 上可用 `npx tauri build --bundles nsis` 打出当前功能的安装包；本仓库 macOS 仍不把 Windows 写成已支持。

**Architecture:** 现有 `desktop/` Tauri 2。bundle 默认仍是 macOS `dmg`。Windows 用 `--bundles nsis` 覆盖。Actions 仅 `workflow_dispatch`，不作为合并红灯。

**Tech Stack:** Tauri 2 NSIS、GitHub Actions `windows-latest`。

## Global Constraints

- 本地优先；启动器仍只搜本地。
- 开机启动与托盘在 Windows 上仍不得声称已生效。
- 本仓库 `backend/` 仍是预发；安装包不捆绑管理端。
- 没有商店包、没有公开下载声明。

---

### Task 1: NSIS 配置与构建说明

**Files:**

- `desktop/src-tauri/tauri.conf.json`
- `desktop/src-tauri/icons/icon.ico`
- `desktop/package.json`
- `.github/workflows/desktop-windows.yml`
- `docs/how-to/local-dev.md`
- `docs/INDEX.md`

- [x] **Step 1: 生成 `icon.ico`，登记 NSIS 构建命令**
- [x] **Step 2: 本机是 macOS，不交叉编译出安装包；不得把未跑过的平台勾成通过**
- [x] **Step 3: `./scripts/docs-check`**
