# 附件传输

状态：P1a 私有附件接口已实现并本机验收；同步与投稿附件引用尚未接通。

## Requirements

私有上传、所有权检查、传输与配额 MUST 满足[私有附件实施计划](../../plans/2026-09-08-private-media.md)的 Given/When/Then 场景。上传到服务器不等于提交审核或公开。所有内容下载 MUST 经 API 验证会话与所有者；不得对私有文件返回匿名可用的预签名链接。

既有本地附件边界见[本地库](../library/spec.md)，本轮不静默改变自动同步和投稿行为。元数据字段通过追加迁移保留原记录；不清空数据库或对象存储。

## 测试映射

`backend/src/media_tests.rs` 覆盖鉴权、所有者隔离、存储往返与故障；`desktop/src/platform/squareContract.test.js` 校验 API 鉴权合同。
