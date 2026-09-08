# 关闭本仓库 WorkLog 提交门禁

## 用户决定与范围

用户明确要求永久关闭或删除当前 WorkLog 评审拦截。检查确认本机 `.git/hooks/pre-commit` 仅包含 `WORKLOG_REVIEW_GATE`，未配置其他 hooksPath。

## 操作与验收

1. 将该文件改名为非有效钩子名 `pre-commit.worklog-disabled-20260908`，保留可恢复备份。
2. 不修改全局 Git 设置、不删除评审记录、不关闭测试、文档检查或 CI。
3. GIVEN 没有新的 IDE 评审批准 WHEN Git 执行 pre-commit THEN 不再被 WorkLog 拦截。
4. GIVEN 原有已暂存进展 WHEN 完成本次文档更新 THEN 正常提交保存，不自动推送。

持久策略见[测试门禁](../reference/test-gates.md)。此决定不授权插件日后重新安装该门禁。
