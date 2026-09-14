# Windows Skill 详情路径修复

| 字段 | 值 |
|---|---|
| 状态 | 已修复并公开发布 beta.9（2026-09-14） |
| 问题 | Windows 扫描成功，点击本机 Skill 详情时报「函数不正确。(os error 1)」 |
| 范围 | 原生绝对路径逐级检查、详情回归、beta.9 双平台补丁发行 |

## 实施方案

1. 为扫描后的详情读取增加回归，覆盖中文/空格目录、只读插件缓存、规范化路径和文件内容一致性；先在 Windows CI 运行，记录失败位置。
2. 修正逐级链接检查：Windows Prefix 只用于组装路径，等 RootDir 组成完整根路径后再查询元数据；保留所有实际目录的链接检查和越界拒绝。
3. 运行 Windows 与 macOS 原生测试、文档与发行预检；版本升级为 beta.9，构建同源双平台补丁包。
4. 校验安装包、更新签名、哈希与公开更新清单，发布后记录证据。

## 验收

- Given 已扫描出普通或只读缓存 Skill，路径含中文和空格 When 点击详情 Then 正常返回正文与完整文件清单。
- Given Windows canonicalize 返回扩展长度绝对路径 When 检查并读取 Skill Then 不对不完整的驱动器前缀执行元数据查询，不出现 os error 1。
- Given 链接目录或越界路径 When 尝试可写包操作 Then 原有拒绝规则继续生效。

## 证据

- 回归先行提交 `a7a9f3c`：[Windows CI 34798587770](https://github.com/sutao2/CueTuck/actions/runs/34798587770) 原实现 108 项通过，新增详情测试唯一失败，`tests.rs:38` 返回 `Incorrect function. (os error 1)`，与用户截图一致。
- 修复仅延后 Windows Prefix 的元数据查询，完整根路径及下级目录继续检查。
- macOS 原生回归 113 项通过、6 项显式忽略，含新增扫描到详情测试；首次沙箱运行中的本机 HTTP/特殊文件测试被环境限制，放行后全量通过。
- 修复提交 `c0c586d`：[Windows CI 34798758264](https://github.com/sutao2/CueTuck/actions/runs/34798758264) 109 项通过、6 项显式忽略，同一详情回归转绿；NSIS 安装、启动 10 秒与卸载通过。
- [完整回归 34798758209](https://github.com/sutao2/CueTuck/actions/runs/34798758209) 全部通过，包括前端单元/构建、浏览器交互、后端和原生/MCP。
- beta.9 发行预检通过；macOS arm64 构建、codesign 严格校验、DMG 校验与双平台更新签名/篡改拒绝通过。
- [beta.9 公开发行](https://github.com/sutao2/CueTuck/releases/tag/v0.1.0-beta.9)：7 个资产大小与 SHA256、匿名预览发现、公开 latest.json 字节一致性及标签均核验通过。双平台源码与标签同为 `c0c586df2868224a0aec9b0cf84a888f9053af64`。
- Windows EXE SHA256：`28024d957cbd9649ab1b062ca7ef3b97c6adca7527d4b6aa51c9cf5d94dbcf1b`。版本从 beta.8 到 beta.9 的预览通道升级比较通过。
- 不涉及服务端部署、Skill 内容迁移或本地数据标识变更。macOS 为 ad-hoc 签名，Windows 无 Authenticode；本轮验证原生路径与安装启动，不声称已操作用户的 Windows 实机。
