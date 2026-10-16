# beta.34 内容版本管理发行

## 计划与验收

1. 统一版本 0.1.0-beta.34，同源构建 macOS Apple Silicon 和 Windows x64，运行完整 CI。
2. 验证 Windows 安装/启动/窗口操作/卸载及便携包；Mac 镜像、固定签名、隔离安装及启动。保留用户应用与数据。
3. 服务器部署前备份数据库和部署材料，使用新镜像标签；迁移只增补版本表，核验旧公开内容和新历史接口，更新审核后台。
4. 通过后发布 prerelease 和签名更新清单，切换官网；验证匿名附件、旧版发现新版与同版不提示。

## 场景

- Given beta.33 及旧数据 When 安装新版 Then 原 identifier、数据路径和签名身份保持，缺少来源基准的下载不被覆盖。
- Given 双平台产物 When 发布 Then 来源提交一致且安装验收通过，签名与哈希相符。
- Given 旧线上内容 When 版本迁移 Then 原 ID/内容/收藏/统计保持，历史 API 可读；保留数据库备份及旧镜像回退点。
- Given beta.33 When 检查更新 Then 发现 beta.34；同版不提示，稳定通道排除 beta。

## 状态

已于 2026-10-16 发布部署为 beta.34。Mac 公证与 Windows 发布者签名边界沿用现有预览版。

## 首轮验收修正

完整 CI 发现新增版本/撤回路由未补入 OpenAPI 及“已撤回”缺少英文词条。补齐合同与翻译后，从同一修正提交重新构建双平台包，不绕过门禁。专门复验作者审核更新、下载基准、并发修改、事务备份恢复、合集成员和 Skill 来源更新。

部署构建同时发现审核后台共享 SearchableSelect 的翻译依赖未被 Docker 白名单复制，补齐两个依赖文件；接口鉴权映射测试同步新增三个路由，保持逐项核验。

## 最终验收与部署

- 源提交 `3574c750ab5e44deadd4c6d8438d70392de006b5`；[公开发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.34)，prerelease=true。
- [完整回归](https://github.com/sutao2/CueTuck/actions/runs/38846096445) 全部通过，包含三前端、后端、Mac 原生、MCP、客户端/审核后台浏览器；[Windows 验收](https://github.com/sutao2/CueTuck/actions/runs/38846096356) 通过 145 项原生测试、NSIS 构建、便携包解压启动、安装启动、窗口操作与卸载。
- 本机专项复验：后端 202 项、原生 151 项、更新相关界面/传输 41 项通过；完整前端门禁通过。覆盖作者待审更新、同 ID 审核发布、幂等/过期请求、并发本地修改拒绝、事务备份与失败回滚、恢复副本、合集稳定成员与本地修改保留、Skill 安装来源与备份。写入测试均为隔离数据。
- Mac 最终 DMG 镜像、固定证书、临时目录安装启动及真实更新页面检查通过；原本地库 13 条提示词（含删除记录）、1 个合集、41 个分类、18 个附件逐行保持不变。线上切换后真实安装包读取旧下载的广场 v1，明确显示版本未知并禁用直接替换；未对用户内容执行替换或重新投稿。
- API/审核后台镜像为 `cuetuck-api:beta34-3574c75` / `cuetuck-admin:beta34-3574c75`。备份位于服务器 `/opt/promptark/backups/beta34-3574c75`，包括数据库、媒体和加密配置/密钥；加密口令单独受限保存。旧镜像及 compose.override.yml 保留，失败切换自动恢复旧配置。迁移前后 square_items、publications、skill_publications 的行数和完整数据指纹一致；初始化 22395 个提示词、2 个合集和 2 个 Skill 的 v1，未改写旧内容。
- 公开提示词/合集正文和附件基线一致，历史接口可读；两份社区 Skill 包版本与历史一致，API/审核后台健康。
- 官网目录 `/opt/cuetuck-website-preview/releases/beta34-3574c75`，旧 `beta33-0a9b093` 保留回退；公网 site.js 与本地构建字节一致。
- 八份发行资产回读 SHA-256 相符；Mac DMG、Windows NSIS、portable ZIP 和 latest.json 匿名 HTTP 200。按客户端选择逻辑验证 beta.32/beta.33 发现 beta.34，同版不提示、稳定通道排除 beta；公开清单的验签下载与 Mac 隔离安装通过。
- 证据位于 `output/releases/v0.1.0-beta.34/`。内容更新需要手动检查，尚无后台主动通知/新版角标；Windows 功能保护以原生及界面自动化测试覆盖，未宣称人工逐页执行完整作者发布流程。
