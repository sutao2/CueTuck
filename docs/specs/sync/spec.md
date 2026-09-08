# 个人库同步

| 字段 | 值 |
|---|---|
| 状态 | 已指定，立即同步、较新者胜、保留本地与仅 Wi-Fi 跳过封面已实现 |
| 关联 | [设置](../settings/spec.md) · [web](../web/spec.md) · [ADR 0014](../../architecture/decisions/0014-full-product.md) |

## Purpose

登录后把本机提示词、合集、分类与非密钥设置同步到账号库。断网时本机库仍全部可用。

分类的删除墓碑及内容归属处理见[分类规格](../categories/spec.md)；同步不得把已删除分类重新展示或保留指向它的分类引用。

## Requirements

### Requirement: 立即同步

已登录时「立即同步」MUST 向预发 API 推拉变更。MUST NOT 在未登录时调用同步接口。MUST NOT 把 Refresh 写入 Web Storage。

个人库同步和队列送达 MUST 分阶段反馈：仍有当前账号的任务时不得宣称全部完成，提供仅队列重试。同步中不得关闭设置或重复提交；账号变化后不得显示上一账号的成功。具体场景见 [P3c 计划](../../plans/2026-09-08-publish-sync-feedback.md)。

原生 HTTP 请求 MUST 有界（连接 10 秒，总请求默认 30 秒；附件可显式延长至 45 秒），网络失败后可重新同步；已推送记录不得因拉取失败而丢失。验证见 [P4–P6 计划](../../plans/2026-09-08-local-release-readiness.md)。

#### Scenario: 登录后立即同步

- GIVEN 用户已登录且本机有一条「本地仍在」
- WHEN 用户点立即同步
- THEN 账号库出现该标题
- AND 启动器仍只搜本机 SQLite

#### Scenario: 未登录不请求

- GIVEN 用户未登录
- WHEN 用户点立即同步
- THEN 打开登录
- AND 不出现已同步

### Requirement: 冲突

默认 MUST 采用较新 `updated_at`。用户选择保留本地时 MUST 不覆盖该条本机正文。MUST NOT 把尚未存在于本机的远端条目拦掉。

#### Scenario: 较新者胜

- GIVEN 本机与远端同一 id 且远端 `updated_at` 更晚
- WHEN 立即同步
- THEN 本机正文为远端版本

#### Scenario: 保留本地

- GIVEN 本机与远端同一 id 且远端 `updated_at` 更晚，且用户选择保留本地
- WHEN 立即同步
- THEN 该条本机正文仍是本机版本
- AND 远端独有条目仍写入本机

### Requirement: 仅 Wi-Fi 同步图片

开启「仅 Wi-Fi 下同步图片」且网络不是 Wi-Fi（含无法判定）时，立即同步 MUST 跳过合集封面，MUST 仍推送提示词正文与合集标题。MUST NOT 用空封面覆盖远端已有封面。开关关闭或网络判定为 Wi-Fi 时 MUST 仍推送本机封面。

#### Scenario: 非 Wi-Fi 跳过封面仍同步正文

- GIVEN 开关已打开、网络不是 Wi-Fi，本机合集有封面且本机有提示词正文
- WHEN 立即同步
- THEN 推送不含该本机封面
- AND 合集标题与提示词正文仍被推送

#### Scenario: 跳过时保留远端封面

- GIVEN 开关已打开、网络不是 Wi-Fi，远端同一合集已有封面
- WHEN 立即同步
- THEN 推送的封面仍是远端已有封面
- AND 合集标题为本机标题

#### Scenario: Wi-Fi 下仍推送封面

- GIVEN 开关已打开且网络判定为 Wi-Fi
- WHEN 立即同步
- THEN 本机合集封面被推送

#### Scenario: 关闭开关时未知网络仍推送封面

- GIVEN 开关关闭且网络无法判定
- WHEN 立即同步
- THEN 本机合集封面被推送

### Requirement: 浏览器账号库

浏览器已登录后 MUST 读写账号库。MUST NOT 声称打开了桌面 SQLite 文件。

#### Scenario: 浏览器登录后同一标题

- GIVEN 桌面已把「本地仍在」同步到账号
- WHEN 用户在浏览器登录同一账号
- THEN 列表可见该标题
- AND 说明不出现「已写入本机 SQLite」

### Requirement: 完整往返与删除

同步 MUST 包含自定义分类、合集、成员归属、模型、来源、更新时间与软删除标记。远端记录 MUST 按分类、合集、提示词依赖顺序在一个 SQLite 事务中落地；非法分类关联 MUST 失败而不是写出孤儿记录。新时间戳采用 Unix 毫秒；旧 Unix 秒值 MUST 在比较时归一化。保留本地策略不得覆盖已有提示词（含软删除记录）。设置仅同步明确白名单的外观/模型偏好，不同步密钥、网络地址或本机队列。

#### Scenario: 第二台设备恢复库

- GIVEN 设备 A 的自定义分类下有合集和带模型的成员提示词
- WHEN B 同步该账号库
- THEN 分类、合集、成员与模型完整出现
- AND A 的软删除同步到 B 后，默认列表不再出现该提示词

#### Scenario: 旧秒值与毫秒值

- GIVEN 较早 Web 毫秒版本与较晚桌面秒版本
- WHEN 合并同一 id
- THEN 较晚桌面版本胜出，不按字符串长度或字典序误判

回归：SQLite `sync_round_trip`、`librarySync.test.js` 设备恢复与删除。

#### Scenario: 恢复 Wi-Fi 后补齐封面

- GIVEN 非 Wi-Fi 同步已传输合集标题，但跳过了本机或远端封面
- WHEN 网络恢复 Wi-Fi 并再次同步
- THEN 之前跳过的封面可上传或下载，不被相同更新时间拦掉
- AND 非 Wi-Fi 阶段不清空本机原封面

## 测试映射

### 私有附件手动补齐

附件同步 MUST 遵循 [P1b 实施计划](../../plans/2026-09-08-private-asset-sync.md) 的场景。只有本次显式勾选才传输文件；默认正文同步不包含附件字节且不能清空云端引用。附件采用补齐合并，不传播删除；本机文件不得因远端未包含而被删除。失败不得落地部分附件或声称完成。

| 场景 | 测试 |
|---|---|
| 登录后立即同步 | `WorkbenchShell.spec.js` pushes the local library to the account when signed in and syncing now；`librarySync.test.js` puts the local prompt onto the account library when signed in |
| 未登录不请求 | `WorkbenchShell.spec.js` shows sync rows without requesting the backend；`librarySync.test.js` does not call the library API when signed out |
| 较新者胜 | `librarySync.test.js` applies the remote body when the remote updated_at is newer；`backend` `newer_updated_at_wins_when_putting_library_changes`；`desktop/src-tauri` `newer_remote_body_replaces_older_local_prompt` |
| 保留本地 | `librarySync.test.js` keeps the local body when keep-local is chosen and remote updated_at is newer；`WorkbenchShell.spec.js` keeps the local body when keep-local is selected before syncing |
| 非 Wi-Fi 跳过封面仍同步正文 | `librarySync.test.js` skips collection covers when wifi-only is on and the network is not wifi |
| 跳过时保留远端封面 | `librarySync.test.js` keeps remote covers when wifi-only skip would otherwise wipe them |
| Wi-Fi 下仍推送封面 | `librarySync.test.js` sends collection covers when wifi-only is on and the network is wifi |
| 关闭开关时未知网络仍推送封面 | `librarySync.test.js` sends collection covers when wifi-only is off even if the network is unknown |
| 浏览器登录后同一标题 | `web/src/WebApp.spec.js` shows the account library title after login without claiming sqlite |
| 变更推拉 API | `backend` `put_then_get_library_changes_for_signed_in_account` |
