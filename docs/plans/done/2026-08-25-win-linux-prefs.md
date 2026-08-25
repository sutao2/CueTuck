# 完成记录：Windows / Linux 开机启动与托盘

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-win-linux-prefs.md](../2026-08-25-win-linux-prefs.md) |

## 退出标准

- [x] Windows 上开机启动写入当前用户 Run 键；托盘偏好可保存；不声称 NSIS 已验证
- [x] Linux 上开机启动写入 XDG autostart；托盘偏好可保存；`detectHost` 识别 Linux
- [x] `release-qa` 的 Windows / Linux 行仍为跳过
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
109 passed

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked --offline commands::desktop
writes_and_removes_launch_agent / windows_startup_record / linux_autostart_entry passed

./scripts/docs-check
docs-check 通过（152 个 Markdown 文件）。
```

未在 Windows 或 Linux 真机上手工验证开机启动与托盘。发行 QA 不得勾通过。未重跑 Windows NSIS 工作流。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：Windows / Linux 偏好已关闭；下一步预发账单

## 未做 / 下一里程碑带走

- 预发账单
- 商店上架、生产托管、Windows NSIS 额度
- Windows / Linux 真机发行 QA
