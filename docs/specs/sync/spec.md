# 个人库同步

| 字段 | 值 |
|---|---|
| 状态 | 已指定，尚未实现 |
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

默认 MUST 采用较新 `updated_at`。用户选择保留本地时 MUST 不覆盖该条本机正文。

#### Scenario: 较新者胜

- GIVEN 本机与远端同一 id 且远端 `updated_at` 更晚
- WHEN 立即同步
- THEN 本机正文为远端版本

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
| 较新者胜 | 未开始 |
| 浏览器登录后同一标题 | 未开始 |
| 变更推拉 API | `backend` `put_then_get_library_changes_for_signed_in_account` |
