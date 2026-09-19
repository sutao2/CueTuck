# beta.20 使用体验与数据恢复发行

状态：已发布并完成官网切换（2026-09-19）。

## 步骤与验收

1. 统一版本为 0.1.0-beta.20，冻结同一源码构建 macOS arm64 与 Windows x64；保留应用标识、数据目录、固定 Mac 签名身份及更新密钥。
2. 验证远程完整回归、Windows 安装启动卸载、Mac 包完整性和双平台更新签名；本地功能验收见[使用体验与可恢复性](2026-09-19-usability-safety.md)。
3. 将完整七份发行附件上传草稿，核对 SHA-256 后公开预览发行；切换官网下载入口并验证公开更新清单、下载及 API 健康。保留官网上一版以便回退。

- Given 既有用户库与设置 When 安装新版 Then 使用原有标识和目录，不清理或迁移真实用户数据。
- Given 两平台包及 CI 完成 When 发布 Then 资产源码一致、签名校验有效、更新清单完整；任何一项失败不公开不完整发行。
- Given 新官网版本已部署 When 公网核验失败 Then 使用保留的上一版目录回退。
- Given 本次仅客户端功能改变 When 部署 Then 后端及数据库保持现有运行版本，只核验健康。

本版仍为预览版，固定 Mac 自签名不等于 Apple 公证，Windows 未配置 Authenticode。安装 smoke 与完整实机交互验证分别记录，不互相替代。

## 发行验收

- 冻结源码与标签：`b3e11a351ff2d042fafc73152251896311515f96`；公开预览发行 [v0.1.0-beta.20](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.20)。七份附件的 GitHub SHA-256 全部与本地一致。
- [完整远程回归](https://github.com/sutao2/CueTuck/actions/runs/35436604407)通过：三前端测试与构建、桌面/管理端浏览器流程、后端、Mac 原生及 MCP。既有显式外部测试的忽略项不算通过。
- [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/35436604413)通过：原生测试、x64 NSIS、安装、启动 10 秒与卸载；build-info 的源码、架构、生产 API 和安装包哈希一致。
- Mac arm64 DMG 完整性、只读挂载、版本、生产 API、二进制一致性及固定证书严格签名校验通过；beta.19 与 beta.20 的签名 DR 相同。
- 两平台更新签名验证通过，篡改字节均被拒绝。Mac 真实更新器通过 localhost 重放签名包，完成下载进度及临时隔离应用安装；未替换用户已安装应用或真实用户库。
- 公开发行、七份附件及 latest.json 均可匿名访问，更新清单与本地一致；两平台安装包匿名链接 HTTP 200，发行标签指向冻结源码。
- 官网已切换至 `/opt/cuetuck-website-preview/releases/beta20-b3e11a35`；公网 site.js 与本地 SHA-256 一致，官网和管理端 HTTP 200，API 的 Postgres、Redis、MinIO 全部健康。Nginx/Compose 配置保持一致，旧目录 `beta19-6b47514b` 保留用于回退；后端及持久卷未改动。
- 本机验收文件位于 `output/releases/v0.1.0-beta.20/`，不提交安装包、私钥或环境秘密。源码、版本及发行记录分批提交。
