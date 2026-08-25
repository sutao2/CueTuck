# 账号与广场剩余行

> **给 Agent：** 不是新里程碑。把设置里作者主页、我的发布、下载保留作者接到现行 API。不要做云同步引擎。不要上架商店。启动器仍不请求广场。

**Goal:** 已登录用户能编辑显示名、看到自己的投稿、按开关在下载副本上展示作者；未登录不能改资料。

**Architecture:** `GET/PUT /v1/me`；`GET /v1/publications/mine`；投稿带 `author_email`。本机设置键 `keep_author_on_download`。设置页登录仍走已有登录弹窗。

**Tech Stack:** 现有 Axum、桌面设置弹窗、SQLite settings。

## Global Constraints

- 本地优先。启动器与 MCP 不请求广场。
- 不得出现 QQ / LinuxDo。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 下载保留作者

**Files:** `desktop/src/components/SettingsModal.vue` · `desktop/src/platform/library.js` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（开关打开时下载副本含作者；关闭时新下载不含）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 本机键 `keep_author_on_download`；下载写入展示用作者字段，不改正文**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 我的发布

**Files:** `backend/src/` · `desktop/src/components/SettingsModal.vue` · `docs/specs/settings/spec.md` · `docs/reference/openapi/square.yaml`

- [ ] **Step 1: Write the failing test**（已登录且有 pending 时设置页列出该条与状态）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 投稿存 `author_email`；`GET /v1/publications/mine`**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 3: 作者主页

**Files:** `backend/src/` · `desktop/src/components/SettingsModal.vue`

- [ ] **Step 1: Write the failing test**（登录后保存显示名，再打开仍在；未登录不能保存）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: `GET/PUT /v1/me` 显示名与简介**
- [ ] **Step 4: 测试 PASS；关闭本计划 done 记录**
- [ ] **Step 5: `./scripts/docs-check` 并提交**
