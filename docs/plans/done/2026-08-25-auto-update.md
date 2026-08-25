# 完成记录：自动更新安装

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-25 |
| 里程碑 | 无（M9 之后切片） |
| 计划 | [2026-08-25-auto-update.md](../2026-08-25-auto-update.md) |

## 退出标准

- [x] 检查更新请求 GitHub Releases；空列表说明没有可用更新；不出现已从商店
- [x] 读取失败说明检查失败，不写成没有可用更新
- [x] 自动下载打开且当前通道有包时通过 updater 排队安装；稳定/预览通道分开
- [x] 启动器不负责安装更新
- [x] 桌面测试与 docs-check 通过

## 命令与结果

```text
cd desktop && npm test
106 passed

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked
36 passed (1 ignored)

./scripts/docs-check
docs-check 通过（151 个 Markdown 文件）。
```

仓库 `sutao2/PromptArk` 仍是私有仓库时，未认证检查会得到 404，界面为检查失败而不是没有可用更新。未对真实签名发行物做手工安装 smoke。桌面包 `package.json` 版本 `0.0.0` 与 Cargo / `tauri.conf.json` 的 `0.1.0` 仍不一致。

## 文档

- 更新的规格：settings
- 更新的 INDEX：是
- status.md：自动更新安装已关闭；下一步 Windows / Linux 开机启动与托盘

## 未做 / 下一里程碑带走

- Windows / Linux 开机启动与托盘、预发账单
- 商店上架、生产托管、Windows NSIS 额度
- 公开 Releases 或带令牌的检查，才能读到真实发行物
- 用同一版本号对齐桌面包与 Tauri 构建
