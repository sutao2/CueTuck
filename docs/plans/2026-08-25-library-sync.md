# 个人库云同步

> **给 Agent：** 不是新里程碑。先读 [sync 规格](../specs/sync/spec.md) 与 [full-product design](../changes/full-product/design.md)。队首 account-surface 关闭后再把本文件改为现行并补逐步任务。启动器与 MCP 仍只碰本机 SQLite。

**Goal:** 登录后立即同步能把本机库与账号库对齐；浏览器登录后看到同一账号库；冲突默认较新者胜。

**Architecture:** `/v1/library/changes` 推拉。桌面写 SQLite 再同步。浏览器已登录走账号库。

**Tech Stack:** Axum、Postgres `promptark`、MinIO、 rusqlite。

## Global Constraints

- 本地优先。断网本机仍可用。
- 不得把 Refresh 写入 Web Storage。
- 不得声称打开了另一台机器的 SQLite 文件。
- 没有本计划之外的应用文件。
- 成为队首后再写逐步 Task；未成为队首禁止写应用代码。
