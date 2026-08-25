# 另立项（无逐步计划）

| 字段 | 值 |
|---|---|
| 状态 | 目标 |
| 关联 | [status](status.md) · [ADR 0014](../architecture/decisions/0014-full-product.md) |

这些项**没有**逐步任务，也还不能诚实完成。禁止假装已接通。

| 项 | 为什么还不能做完 | 动之前 |
|---|---|---|
| 商店上架 | 需要开发者账号与商店审核，仓库里做不出「已上架」 | 人提供账号与审核证据 |
| 生产托管 | 预发仍是本机 `promptark` 库 | 人指定主机与域名后再立项 |
| Windows NSIS 验证 | 工作流已在；失败因 GitHub 额度 | 额度恢复后跑 Actions；通过前 `release-qa` 不得勾 Windows |
| 原生移动端 | 产品明确不做 | 新变更 |

个人库同步、自动更新、预发账单、Windows/Linux 桌面偏好已进入 [program.md](program.md) 切片队列，不再放本表。
