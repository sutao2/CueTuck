# 附件传输

状态：P1a 私有接口、P1b 手动私有附件同步、P1c 单条投稿及 P1d 合集成员公开附件已实现。

## Requirements

私有上传、所有权检查、传输与配额 MUST 满足[私有附件实施计划](../../plans/2026-09-08-private-media.md)的 Given/When/Then 场景。上传到服务器不等于提交审核或公开。私有下载 MUST 经 API 验证会话与所有者；不得对私有文件返回匿名可用的预签名链接。

稿件范围的审核读取和已批准公开读取 MUST 满足 [P1c 计划](../../plans/2026-09-08-publication-assets.md)，不扩大原有私有下载权限。公开字节不走存储直链，且不自动执行文档。

合集成员文件关联、整稿限制及原子下载 MUST 满足 [P1d 计划](../../plans/2026-09-08-collection-assets.md)，沿用稿件清单授权，不扩大作者或审核者的私有访问权限。

既有本地附件边界见[本地库](../library/spec.md)，本轮不静默改变自动同步和投稿行为。元数据字段通过追加迁移保留原记录；不清空数据库或对象存储。

手动私有引用、重试复用与客户端补齐 MUST 满足 [P1b 计划](../../plans/2026-09-08-private-asset-sync.md)。云端引用不能指向其他账号或未完成对象。正文旧客户端省略引用时 MUST 保留已有引用；文件字节不进入账号变更 JSON。

## 测试映射

`backend/src/media_tests.rs` 覆盖鉴权、所有者隔离、存储往返与故障；`desktop/src/platform/squareContract.test.js` 校验 API 鉴权合同。
