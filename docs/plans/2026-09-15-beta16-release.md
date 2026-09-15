# beta.16 昵称弹窗修复发行

状态：已公开发布并部署官网下载入口。

1. 将桌面版本统一为 0.1.0-beta.16，纳入已通过回归的启动昵称弹窗修复；准备官网新版下载入口，公开发行后上线。
2. 双平台使用固定源码与正式 API，沿用已有更新签名。执行远程全量 CI、Windows 构建与安装测试、本机 macOS arm64 构建。
3. 验证版本、源码、签名、篡改拒绝及附件校验和。先上传草稿，全部通过后公开预览版，再验证匿名更新发现。
4. 更新官网发行和下载链接，保留静态站点旧版本可回滚，验证 API 和管理端健康。无需变更后端或账号数据库。

- Given beta.15 客户端 When 检查预览更新 Then 可发现 beta.16 的可信签名包。
- Given 任一构建或验收失败 When 准备发布 Then 不公开不完整发行。
- Given 官网下载页面 When 新发行公开 Then 下载链接指向实际存在的 beta.16 macOS / Windows 包。

## 发行验收

- 固定发行源码与标签：`00a5f2f8f007847712616042300a2486dfdf473b` / `v0.1.0-beta.16`。双平台均为此版本与正式 API。
- [全量 CI](https://github.com/sutao2/CueTuck/actions/runs/34944632984) 成功，包括前端、浏览器、后端、原生客户端和 MCP；昵称修复本机桌面全量 602 项回归通过。
- [Windows CI](https://github.com/sutao2/CueTuck/actions/runs/34944632812) 成功，完成安装、启动 10 秒与卸载；构建记录的源码、API、x64 架构和附件哈希一致。
- macOS arm64 DMG 完整性、只读挂载后的版本与正式 API、严格代码签名，以及安装包/更新包可执行文件一致性通过。仍为 ad-hoc 签名，未做 Apple 公证；Windows 未做 Authenticode 签名。
- 两平台更新包通过既有公钥验签，篡改后的字节被拒绝。清单及 SHA256SUMS 一致，七个远端发行附件哈希与本机匹配。
- [公开预览版](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.16) 已可匿名发现；按客户端预览渠道规则，beta.15 可发现 beta.16，公开更新清单可下载。
- 官网下载入口已部署到 `/opt/cuetuck-website-preview/releases/20260915-beta16-1`，原 `20260915-launcher-1` 保留。Compose healthy，API 的 Postgres/Redis/MinIO 均健康，管理端正常。
- 本次发布客户端与官网下载入口；用户需在设置中升级。未替换本机安装，未修改业务数据库。
