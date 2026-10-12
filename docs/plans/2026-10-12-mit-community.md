# MIT 许可证与社区入口

状态：已验收。

## 范围

按用户要求采用 MIT，在根目录 LICENSE 保存完整许可文本；各自有 npm/Rust 包声明 MIT。第三方依赖、社区样例及素材已有的许可和归属保持有效。

参考 InkNote 的 README，七种语言首页增加 MIT 徽章、许可证说明与 LinuxDO 社区交流入口，链接社区首页，不宣称官方合作或代发社区帖子。

## 验收场景

- Given 阅读任意语言 README When 点击 MIT 徽章或许可链接 Then 打开根目录 LICENSE。
- Given 阅读任意语言 README When 点击 LinuxDO Then 打开 https://linux.do/。
- Given 检查自有包元数据 When 读取 npm 清单、锁文件根包及 Cargo 清单 Then license 均为 MIT。

## 验证

七份 README 相对链接有效，社区入口一致；9 处包许可声明均为 MIT。`python3 scripts/docs-check` 通过（362 份 Markdown），差异空白检查通过。此次仅修改许可证、项目介绍与元数据。
