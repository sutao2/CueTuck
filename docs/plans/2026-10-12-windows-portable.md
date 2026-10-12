# Windows 免安装 ZIP

状态：已验收。

## 范围与步骤

1. 为现有 Windows 构建增加 `CueTuck_<version>_x64-portable.zip`，复用同次 release 构建的主程序，附 MIT LICENSE 和中英使用说明。
2. 在 NSIS 安装测试之前，从 ZIP 解压到含空格的临时目录启动并验证窗口；记录 SHA-256、源提交和 smoke 结果，随现有 Windows artifact 上传。
3. 更新发行合同与 QA，运行文档检查并通过真实 Windows CI 验证。

此处 portable 指免安装分发：程序继续使用 Windows 用户数据目录和系统凭据库，与安装版共用数据；不承诺 U 盘携带全部数据。需要系统 WebView2 Runtime。保持免安装方式时退出应用、下载新版 ZIP 替换程序；内置更新仍使用 NSIS 安装流程。此次不发布新版本或改变 updater 清单。

## 验收场景

- Given Windows 构建成功 When 打包 Then ZIP 包含主程序、许可证和使用说明，版本与 Tauri 配置一致。
- Given 新解压目录且本轮 NSIS 尚未运行 When 启动 ZIP 内主程序 Then 应用运行至少 10 秒并出现主窗口。
- Given Windows artifact When 校验 ZIP Then SHA256SUMS-windows 和 build-info.json 中哈希与实际文件相符，exe 与本轮构建一致。
- Given 便携版用户 When 阅读包内说明 Then 能了解 WebView2、数据位置、与安装版的共用关系及手动升级方法。

## 验证记录

- 源提交 `1dc0554b8ff8aa0796e8d063e02335dbda6ec40b` 的 [Windows CI](https://github.com/sutao2/CueTuck/actions/runs/38217837929) 成功：原生测试、NSIS 构建、ZIP 解压启动、安装版启动/窗口按钮与卸载均通过。
- 下载 artifact 回读：ZIP 内仅含 `cuetuck.exe`、`LICENSE`、`README.txt`，归档完整；安装包与 ZIP 的 SHA-256 均匹配 SHA256SUMS-windows，portable 哈希匹配 build-info.json，source_commit 与构建提交一致。
- 本次便携产物：`CueTuck_0.1.0-beta.28_x64-portable.zip`；SHA-256 `68c64c36004214ced0b3282b179e8a2ea1b84aea1a7047e7e46f84955a930c24`。只生成 CI artifact，未添加到现有 GitHub Release。
- `python3 scripts/docs-check`（363 份 Markdown）和 `git diff --check` 通过。
