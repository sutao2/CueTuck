# 自动更新安装

> **给 Agent：** 不是新里程碑。检查更新必须是真请求。没有发行物就说没有可用更新。不得声称已上架商店。队首关闭后再把本文件改为现行并补逐步任务。

**Goal:** 设置更新页能检查 GitHub Releases、展示发行说明、按通道下载并安装；无发行物时不假装已从商店安装。

**Architecture:** Tauri updater 指向本仓库 Releases。自动下载是本机开关。

**Tech Stack:** tauri-plugin-updater、GitHub Releases。

## Global Constraints

- 不得声称 Mac App Store / Microsoft Store 已上架。
- 未成为队首禁止写应用代码。
