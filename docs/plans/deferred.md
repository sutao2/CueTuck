# 另立项（无逐步计划）

| 字段 | 值 |
|---|---|
| 状态 | 目标 |
| 关联 | [status](status.md) · [ADR 0006](../architecture/decisions/0006-plan-altitude.md) |

这些缺口**没有** `YYYY-MM-DD-*.md` 逐步任务。未接受变更提案前禁止写应用代码，禁止假装已接通。

| 项 | 为什么另开 | 动之前 |
|---|---|---|
| 云同步个人库 | 宪法禁止假装；M9 已说明浏览器≠桌面库 | 新变更，不是 web 修修补补 |
| 自动更新安装 | 设置行已占位 | 新变更 |
| 账单 / 商店上架 | 产品明确后期 | 新变更；不得改 README 声称已上架 |
| Windows NSIS 验证 | 工作流已在；失败因 GitHub 账号额度，不是缺代码 | 额度恢复后跑 Actions；通过前 `release-qa` 不得勾 Windows |
| Windows / Linux 开机启动与托盘 | 仅 macOS 已验证 | 目标平台手工 QA，不得提前勾选 |

排在 [program.md](program.md) 切片队列之后。队首四份计划全部关闭后，再决定是否开上述变更。
