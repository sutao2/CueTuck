# beta.15 双平台发行

状态：已发布并验收。

1. 统一桌面版本至 0.1.0-beta.15，纳入 Skills 创建、社区投稿审核与完整包安装、入口去重、广场布局及重复右键菜单移除。
2. 推送固定源码，沿用已有更新签名与正式 API；运行全量 CI，构建 macOS arm64 和 Windows x64。
3. 验证版本、源码、平台、安装烟测、代码签名、更新签名与篡改拒绝，生成清单及校验和。
4. 先上传 GitHub 草稿，资产一致且检查通过后公开 prerelease，核对匿名更新发现并记录验收。

- Given 旧版客户端 When 检查预览更新 Then 能发现 beta.15 对应平台的可信签名包。
- Given 两平台安装包 When 验证 Then 使用相同源码、版本与正式 API。
- Given 任一必要检查失败 When 准备公开 Then 保持草稿直至修复并完成检查。

本次仅发布客户端；已上线的服务端与管理端无需重复部署。macOS 使用 ad-hoc 签名且未做 Apple 公证；Windows 未做 Authenticode 签名。

## 发行验收

- 固定发行源码与标签：`35b01fd9c56867430339ddf340167dc537089608` / `v0.1.0-beta.15`。双平台版本与正式 API 一致。
- [全量 CI](https://github.com/sutao2/CueTuck/actions/runs/34937808642) 全部通过：桌面 598 项、管理端 105 项、网页 33 项测试及构建、浏览器流程、服务端、原生客户端和 MCP 回归。
- [Windows CI](https://github.com/sutao2/CueTuck/actions/runs/34937808593) 成功，完成安装、启动 10 秒和卸载；构建记录中的源码、正式 API、x64 架构及安装包哈希均已核对。
- macOS arm64 版本、深度严格代码签名、DMG 完整性通过；只读挂载后再次确认正式 API、签名及可执行文件与更新归档一致。
- 两平台更新包用既有公钥验签通过，并拒绝篡改字节；清单、签名和 SHA256SUMS 一致。
- [公开 prerelease](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.15) 的 7 个附件远端 SHA-256 与本机一致；匿名访问发行列表及下载更新清单成功，旧版预览渠道可发现 beta.15，标签指向固定发行源码。
- 本次未重新安装本机客户端，未重复部署已上线的服务端与管理端。
