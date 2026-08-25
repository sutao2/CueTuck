# Windows / Linux 开机启动与托盘

> **给 Agent：** 不是新里程碑。写出开机启动与关闭后托盘的行为。未在该 OS 手工验证前，`release-qa` 不得勾选。不得声称 Windows NSIS 已验证。队首关闭后再改为现行并补逐步任务。

**Goal:** Windows 与 Linux 上开机启动、最小化到托盘与 macOS 已有行为对齐；保存失败时开关回退。

**Architecture:** 现有桌面偏好命令按目标 OS 分支实现。

**Tech Stack:** 现有 Tauri 桌面命令。

## Global Constraints

- 未验证平台不得勾选发行 QA。
- GitHub 额度恢复前不重跑 Windows NSIS 工作流，除非人明确要求。
- 未成为队首禁止写应用代码。
