# beta.18 合集与固定签名发行

状态：已发布并部署（2026-09-18）。

1. 版本统一为 0.1.0-beta.18，冻结同一源码供 Mac arm64 / Windows x64 构建；沿用更新密钥、应用 identifier、数据目录和固定 Mac 预览证书。
2. 推送代码并验证完整 CI，构建安装包与签名更新包；验证版本、来源提交、生产 API、Mac 固定签名要求、更新签名和 SHA-256。
3. 备份线上数据库、对象和加密配置；独立构建 API/admin 镜像，切换后检查健康、封面字段和旧内容兼容；失败回退原镜像。
4. 七份附件完整验收后公开 GitHub 预览发行，更新官网入口并核验匿名更新清单。保留旧发行和服务器版本用于回滚。

- Given 合集封面新版客户端 When 发布 Then 线上 API 接受并经人工审核公开。
- Given 任一平台构建失败 When 准备公开 Then 不发布不完整版本。
- Given 旧 Mac ad-hoc 安装 When 首次迁移固定证书 Then 可能需要一次钥匙串授权；不宣称已获 Apple 公证。
- Given 同一固定证书的后续包 When 校验身份 Then DR 和已确认指纹一致。

## 发行验收

- 冻结源码 `fdc284d325602bdee177a875806e3e25dce2609a`，标签及公开预览版：[v0.1.0-beta.18](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.18)。七份附件已逐一核对 GitHub 上传 SHA-256。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/35301345754) 和 [Windows 构建](https://github.com/sutao2/CueTuck/actions/runs/35301345761) 通过。Windows x64 安装、运行 10 秒、卸载通过，build-info 与冻结源码、生产 API 一致。
- Mac arm64 DMG 完整性、只读挂载、版本和二进制一致性通过；固定证书 DR 与既有身份一致。两平台更新签名通过，篡改包被拒绝。
- Mac updater 使用真实签名包在隔离临时应用中完成下载进度与安装验证；该安装验证经 localhost 重放，未替换用户应用。公开 GitHub 发行和 latest.json 匿名读取通过，清单与本地逐项一致，两平台包匿名下载地址均 HTTP 200。
- 线上备份 `/opt/promptark/backups/beta18-fdc284d3` 包含已校验数据库转储、对象、加密配置及 Compose 配置；未覆盖用户资料。
- API/admin 镜像 `cuetuck-api:beta18-fdc284d3` / `cuetuck-admin:beta18-fdc284d3` 已切换且健康，Postgres `publications.cover` 迁移完成。公网 browse 返回 cover/cover_assets，现有 22394 条内容可读；推荐排除 24 条和拼写容错搜索通过，管理端 HTTP 200。
- 官网 `/opt/cuetuck-website-preview/releases/beta18-fdc284d3` 已上线，根域名使用 beta.18 发行及双平台下载链接；旧镜像、站点目录与备份保留用于回滚。
- 仍为无 Apple 公证 / Windows Authenticode 的预览版；旧 ad-hoc 首次迁移可能需重新授权钥匙串，未声称免除系统首次提示。
