# M8 Settings IA Implementation Plan

> **For agentic workers:** 当前实现里程碑是 M8。不要删已有设置行为。不要接 OAuth。不要实现云同步引擎或自动更新安装。启动器不请求广场。

**Goal:** 设置弹窗对齐原型十类；只增加本机行为与诚实占位行，不减少 M2/M4 已交付能力。

**Architecture:** 现有设置弹窗与 `settings` 表。新键只追加。云行不发网络请求。

**Tech Stack:** 现有 Vue / Vitest / Tauri。开机启动与托盘按已验证平台实现；未验证平台不得声称完成。

## Global Constraints

- 本地功能必须离线可用。云端不是使用前提。
- 启动器必须是独立窗口，label 保持 `launcher`。
- 没有本计划任务之外的应用文件。
- 每一任务结束后跑 `./scripts/docs-check`。
- 不把本机 `backend/` 写成生产。
- 不缩小 M7 合同补齐范围。

---

### Task 0: M7 已关闭

- [x] **Step 1:** `docs/plans/status.md` 中 M7 为完成并有 done 记录
- [x] **Step 2:** 才允许开始 Task 1

未完成 Task 0，不得改 `desktop/` 设置代码。

---

### Task 1: 十类导航与诚实占位

**Files:**

- `desktop/` 设置侧栏与十个面板
- 映射 settings「十类都在」「缺少的页不得消失」

- [x] **Step 1: Write the failing tests**
- [x] **Step 2: Run tests — FAIL**
- [x] **Step 3: 最小实现；同步/更新/未接通行不请求后端**
- [x] **Step 4: 测试 PASS；已有打开设置、未实现页、主题测试仍绿**
- [x] **Step 5: `./scripts/docs-check`**

---

### Task 2: 常规本机行

**Files:**

- 开机启动、关闭到托盘、使用后关闭启动器

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；未验证 OS 不得声称已生效**
- [ ] **Step 4: 映射「开机启动」**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 3: 附加快捷键

**Files:**

- 设置快捷键页三行；启动器新建与粘贴最近使用
- 保留唤起快捷键与冲突失败提示

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；启动器仍不请求广场**
- [ ] **Step 4: 映射 settings「新建与粘贴快捷键可见」与 launcher 对应场景**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 4: 数据增加项

**Files:**

- 打开目录、ZIP 完整备份、自动备份
- 保留 JSON 导入预览/导出与库文件备份恢复

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；失败不破坏已有备份文件**
- [ ] **Step 4: 映射「打开目录与 ZIP 行可见」；原备份恢复测试仍绿**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 5: 外观增加项

**Files:**

- 跟随系统、界面语言、双语开关、密度
- 保留浅色/深色及启动器同键

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；关双语不删中英正文**
- [ ] **Step 4: 映射外观增加项与 variables「关闭双语不删正文」**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 6: AI、隐私、账号、网络、更新

**Files:**

- AI 与模型本机目录/标签/建议（不外传正文）
- 隐私：默认不上传、清除使用历史、钥匙串只读状态；匿名统计未接通则标明
- 账号：当前账号接已有登录；作者主页/我的发布未交付则标明尚未提供
- 网络：广场访问开关；代理/同步状态不假装
- 更新：真实版本；检查/自动下载/通道标明尚未提供

- [ ] **Step 1: Write the failing tests**
- [ ] **Step 2: Run tests — FAIL**
- [ ] **Step 3: 最小实现；无 OAuth、无假同步、无假更新结果**
- [ ] **Step 4: 映射对应 settings 场景**
- [ ] **Step 5: `./scripts/docs-check`**

---

### Task 7: 关闭 M8

- [ ] **Step 1:** 勾选 `docs/plans/milestones/m8.md` 退出标准
- [ ] **Step 2:** 写 done 记录，更新 status / roadmap / INDEX
- [ ] **Step 3:** `./scripts/docs-check`
