# 完成记录：M8 设置对齐

| 字段 | 值 |
|---|---|
| 日期 | 2026-08-23 |
| 里程碑 | M8 |
| 计划 | [2026-08-23-m8-settings-ia.md](../2026-08-23-m8-settings-ia.md) |

## 退出标准

- [x] 十类导航有测试映射
- [x] 已有 JSON / 库文件备份 / 浅色深色 / 唤起快捷键测试仍绿
- [x] 新增本机行已接通的有测试映射；云行标明尚未提供
- [x] 同步、更新等云行不请求后端
- [x] done 记录 + status 更新

## 命令与结果

```text
cd desktop && npx vitest run
Test Files  19 passed (19)
Tests  80 passed (80)

cd desktop/src-tauri && unset CARGO_TARGET_DIR && cargo test --locked
33 passed; 0 failed; 1 ignored
（ignored = search_ten_thousand_prompts_bench）

cd backend && unset CARGO_TARGET_DIR && cargo test --locked
21 passed; 0 failed; 1 ignored
（ignored = session_api_postgres_container，无 Docker）

./scripts/docs-check
docs-check 通过（110 个 Markdown 文件）。
```

设置弹窗固定十类。M2/M4 的 JSON 导入预览/导出、库文件备份恢复、浅色/深色、唤起快捷键仍在。本机增加：开机启动（仅已验证 macOS）、托盘、附加快捷键、打开目录、ZIP 备份、自动备份、跟随系统、界面语言、双语开关、密度、本机模型目录、广场访问开关、清除使用历史、当前账号接已有邮箱密码登录。云同步、自动更新安装、OAuth 仍标明尚未提供，不发假请求。启动器仍只搜本地。本仓库 `backend/` 是预发。未打发行标签。

## 文档

- 更新的规格：settings、variables
- 更新的 INDEX：是
- status.md 已改为该里程碑完成：是

## 未做 / 下一里程碑带走

- 密码 KDF、Postgres、OAuth、账单、商店上架
- Windows / Linux 开机启动与托盘尚未验证，不得声称已生效
- 云同步引擎与自动更新安装须另立项
