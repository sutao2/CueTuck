# 桌面 OAuth 浏览器完成页

- 日期：2026-09-13
- 状态：实现、回归及本机部署完成
- 问题：用户授权后浏览器仍停在 Google 加载页；普通 browser 回调成功返回 204，没有新文档，符合该现象。尚未确认用户客户端是否同时等待。

## 切片与验收

1. browser 回调保留原有授权码、邮箱与会话验证，成功保存轮询结果后返回 200 HTML，明确提示授权完成、切回提示方舟和可关闭页面。不伪造系统深链或依赖浏览器自动关闭。
2. 完成页不含凭据或个人资料，不加载外部资源，使用 no-store/no-referrer；JSON、web_message 和管理验证分支保持既有合同。
3. 新旧回调路径、Google/GitHub 均覆盖：无回调 pending，成功完成页，连续轮询可用于 native poll/commit，非法 state 不产生 ready。
4. 跑 OAuth 回归、文档检查与隔离浏览器导航验证；构建并重启 8080 服务，验证健康后提交。真实 Google 授权由用户重试，不代用户同意。

## Given / When / Then

- GIVEN 桌面 browser 流程有有效授权码和 state；WHEN 回调完成；THEN 浏览器收到 200 完成文档，客户端从同一 flow 取回会话。
- GIVEN 完成文档已显示；WHEN 客户端先检查 ready 再提交登录；THEN 两次查询都可取得同一会话，不因浏览器展示或第一次轮询丢失。
- GIVEN 未收到有效回调；WHEN 查询 flow；THEN 保持 pending，不出现成功文档或签发会话。

## 验收记录

- 新增回归先在旧实现失败（实际 204、期望 200），修复后 `cargo test --locked oauth -- --test-threads=4` 共 16 项通过。测试覆盖两家提供商、两条回调路径及连续会话查询。
- 独立 Chrome / Playwright CLI 对导航行为实测：204 后仍在模拟授权页，200 后 URL 变为完成页并显示「授权完成」。使用与服务端相同的 HTML 文件；1200×800 与 375×700 下无横向溢出。该检查未使用真实 Google 账号。
- `cargo build --locked`、`./scripts/docs-check`（270 文档）、`git diff --check` 通过；本机 8080 后端已重启，Postgres、Redis、MinIO 健康。
- 本次未修改客户端；真实 Google 账号授权后的结果由用户重新发起登录验证。

协议依据：[RFC 9110 §15.3.5](https://www.rfc-editor.org/rfc/rfc9110.html#name-204-no-content)。行为见[认证规格](../specs/auth/spec.md)。
