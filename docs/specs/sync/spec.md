# 个人库同步

| 字段 | 值 |
|---|---|
| 状态 | 已指定，立即同步、较新者胜、保留本地与仅 Wi-Fi 跳过封面已实现 |
| 关联 | [设置](../settings/spec.md) · [web](../web/spec.md) · [ADR 0014](../../architecture/decisions/0014-full-product.md) |

## Purpose

登录后把本机提示词、合集、分类与非密钥设置同步到账号库。断网时本机库仍全部可用。

## Requirements

### Requirement: 立即同步

已登录时「立即同步」MUST 向预发 API 推拉变更。MUST NOT 在未登录时调用同步接口。MUST NOT 把 Refresh 写入 Web Storage。

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

## 测试映射

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
