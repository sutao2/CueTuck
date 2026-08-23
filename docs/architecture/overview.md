# 架构概览

| 字段 | 值 |
|---|---|
| 状态 | 现行 |
| 阶段 | M5 广场已落地；M6 合同已接受 |
| 关联 | [数据模型](data-model.md) · [ADR 0008](decisions/0008-m5-backend-contract.md) · [ADR 0009](decisions/0009-m6-admin-console.md) |

## 容器

```mermaid
flowchart TB
  user[用户]
  operator[运营]
  main[主窗口 Vue]
  launcher[启动器窗口 Vue]
  rust[Tauri Rust]
  sqlite[(SQLite)]
  api[本仓库 backend]
  adminWeb[admin-web]
  user --> main
  user -->|全局快捷键| launcher
  operator --> adminWeb
  main --> rust
  launcher --> rust
  rust --> sqlite
  rust -->|广场 / 登录 / 发布| api
  adminWeb -->|/v1/admin| api
```

主窗口与启动器共享本机 SQLite。M5 起 Rust 访问本仓库广场 API；启动器命令不请求广场或管理接口。`admin-web` 是独立浏览器应用，不进桌面安装包，不进 Tauri 窗口。`backend/` 是本机预发，不声称生产。

## 子系统

| 子系统 | 职责 | 第一期 |
|---|---|---|
| 主窗口工作台 | 分类树、卡片、编辑/使用/创建设置弹窗 | 按原型重建 |
| 启动器 | 独立窗口搜索、填写、复制、粘贴 | 移植旧实现 |
| 本地数据 | SQLite schema、工作区、FTS、备份 | 按本仓库数据模型新建 |
| 桌面集成 | 快捷键、托盘、剪贴板、粘贴、权限 | 随启动器一起移植 |
| 云与广场 | 认证、广场、发布 | M5：新合同，见 ADR 0008 |
| 管理台 | 审核、用户只读、运行时开关 | M6：独立合同，见 ADR 0009 |

## 窗口

- `main`：工作台。
- `launcher`：独立启动器。失焦隐藏、唤起保护期、粘贴前交还焦点等行为以旧实现为准。
- 登录在主窗口弹窗完成，不另开窗口。启动器不打开登录。
- 管理端不占用桌面窗口。

## 前端约定

- Vue 3 + Vite + Pinia + Vue Router（若主窗口需要路由；弹窗流可无路由）。
- 文案简体中文，标识符英语。
- 样式跟原型 `styles.css` 的 token，不沿用旧 Atelier Zero 作为主窗口主题。
- 启动器视觉可保持旧启动器可用性，不要求与原型覆盖层一致。

## 广场后端

合同见 [ADR 0008](decisions/0008-m5-backend-contract.md)。不画旧 Spring 路径。运行时是本仓库 `backend/`。

## 管理后端

合同见 [ADR 0009](decisions/0009-m6-admin-console.md)。独立 OpenAPI，前缀 `/v1/admin`。不把管理路径写进 `square.yaml`。
