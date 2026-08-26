# 钥匙串文案诚实标明

> **给 Agent：** 不是新里程碑。浏览器预览没有系统密钥库。设置钥匙串行与登录脚注不得写成 Refresh 已写入本机钥匙串。Tauri 下仍标明本机钥匙串。不得把 Refresh 改存 Web Storage。

**Goal:** 无 Tauri 时标明 Refresh 不进 Web Storage 且不在本机钥匙串；有 Tauri 时仍标明本机钥匙串。

**Architecture:** 只按 `window.__TAURI_INTERNALS__` 切换文案。不改令牌存放实现。

**Tech Stack:** 桌面设置弹窗、登录弹窗、settings 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 浏览器预览不写本机钥匙串

**Files:** `desktop/src/components/` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing test**（无 Tauri 时钥匙串行与登录脚注不含本机钥匙串 / 只写入系统钥匙串；含不进 Web Storage）
- [ ] **Step 2: Run test — FAIL**
- [ ] **Step 3: 按是否 Tauri 切换文案；更新规格场景与测试映射**
- [ ] **Step 4: 测试 PASS**
- [ ] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md` · `docs/specs/settings/spec.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
