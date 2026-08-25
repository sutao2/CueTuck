# 客户端接 Google / GitHub

> **给 Agent：** 不是新里程碑。后端 OAuth 已有。只接线登录弹窗与浏览器收藏。未配置的提供商不得出现。启动器仍不请求广场。

**Goal:** 桌面、浏览器工作台、管理台能用已配置的 Google / GitHub 登录；未配置则只留邮箱密码。浏览器收藏在登录后真正写入账号关系。

**Architecture:** 读 `/v1/session/oauth/providers`。授权用 `response_mode=browser` + `flow_id` 轮询 `/v1/session/oauth/session/{flowId}`。桌面 Refresh 进钥匙串；web 与 admin-web 的 Refresh 不进 Web Storage。

**Tech Stack:** 现有 Vue 登录弹窗、Tauri session 命令、fetch。

## Global Constraints

- 本地优先。启动器与 MCP 不请求广场。
- 不得发假的提供商请求。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 桌面登录弹窗

**Files:** `desktop/src/components/LoginModal.vue` · `desktop/src/platform/session.js` · `desktop/src-tauri/src/commands/session.rs` · `docs/specs/auth/spec.md`

- [x] **Step 1: Write the failing test**（providers 含 google 时登录弹窗有 Google；空列表没有；设置账号页仍无 QQ/LinuxDo）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 列出提供商；OAuth 走 Tauri 打开授权 URL 并轮询；Refresh 进钥匙串**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 浏览器登录与收藏

**Files:** `web/src/` · `docs/specs/web/spec.md`

- [x] **Step 1: Write the failing test**（未登录收藏仍不新增内存条；登录后收藏不新增内存条；Refresh 不进 Web Storage；有 google 时显示按钮）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 内存会话；OAuth 轮询；`PUT /v1/favorites/{id}`**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 3: 管理台 OAuth 按钮

**Files:** `admin-web/src/`

- [x] **Step 1: Write the failing test**（providers 含 google 时有按钮；OAuth 成功不把 Refresh 写入 Web Storage）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 与浏览器相同的轮询；邮箱密码仍可用**
- [x] **Step 4: 测试 PASS；关闭本计划 done 记录**
- [x] **Step 5: `./scripts/docs-check` 并提交**
