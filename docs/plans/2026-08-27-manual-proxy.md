# 手动配置代理

> **给 Agent：** 不是新里程碑。网络页代理仍写跟随系统。空地址 MUST 跟随系统；填写 http(s) 地址后 MUST 让本机 Tauri 请求走该代理。浏览器预览 MUST NOT 声称走该代理。不得接 SOCKS。不得声称商店上架。

**Goal:** 设置可填写 HTTP/HTTPS 代理；空则跟随系统；非法地址不保存；本机请求使用该地址。

**Architecture:** 本机设置 `http_proxy`。解析与校验在 JS 与 Rust 共用规则。Tauri 的 reqwest 客户端读运行时缓存；空不设 Proxy（跟随环境变量）。启动器仍不请求广场。

**Tech Stack:** 桌面设置、httpProxy、Tauri http 模块、settings 规格。

## Global Constraints

- 不得声称商店上架或生产托管。
- 没有本计划之外的应用文件。
- 每个 Task 做完即提交。

---

### Task 1: 空则跟随系统，填写后本机走代理

**Files:** `desktop/src/platform/` · `desktop/src/components/` · `desktop/src-tauri/src/` · `docs/specs/settings/spec.md`

- [x] **Step 1: Write the failing test**（空显示跟随系统；合法地址可持久化；非法地址不保存；文案标明浏览器预览不走代理；Rust 空为 None、http(s) 为 Some、其它方案失败）
- [x] **Step 2: Run test — FAIL**
- [x] **Step 3: 设置输入、解析、本机 reqwest 应用代理**
- [x] **Step 4: 测试 PASS**
- [x] **Step 5: `./scripts/docs-check` 并提交**

### Task 2: 关闭本计划

**Files:** `docs/plans/` · `docs/INDEX.md`

- [ ] **Step 1: Write the failing check**（完成记录与 INDEX 仍把本计划标现行）
- [ ] **Step 2: 关闭 done 记录；队列回到 deferred**
- [ ] **Step 3: `./scripts/docs-check` 并提交**
