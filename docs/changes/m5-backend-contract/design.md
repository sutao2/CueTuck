# 设计：M5 后端合同

对照 [提案](proposal.md)。提案未接受前不按本文建目录或写代码。

## 方案

接受后先追加 ADR（改写合同、邮箱密码、覆盖率），再把覆盖率抄进 [测试门禁](../../reference/test-gates.md) 的 M5 行。旧 ADR 0003 不改写，只在新篇写 M5 决定。

合同原文进 `docs/reference/openapi/square.yaml` 并登记 INDEX。桌面经 Tauri Rust 发 HTTP、持有 Refresh；Vue 只见会话态，不读 Refresh。后端进程放本仓库 `backend/`，新路由，不沿用旧 Spring 路径。实现语言与框架在开工任务里选定，但必须能跑 Testcontainers 并达到提案覆盖率。

广场列表失败或离线时，主窗口停在非阻断说明。启动器命令路径不增加广场请求。

## 文件

| 路径 | 职责 |
|---|---|
| `docs/reference/openapi/square.yaml` | 广场 / 认证 / 发布合同 |
| `backend/` | 服务与 Testcontainers |
| `desktop/src-tauri` | HTTP 客户端、钥匙串、下载写入 SQLite |
| `desktop/src` | 广场空间、登录与发布弹窗；无 Refresh |

## 风险

- 旧后端字段名与本仓库数据模型不一致；禁止静默兼容层当合同。
- 覆盖率未写入门禁就开写服务，会违反 M5 进入标准。
- 浏览器预览没有钥匙串；与现有「仅桌面窗口支持备份」一样，登录令牌只在 Tauri 窗口持久化。
