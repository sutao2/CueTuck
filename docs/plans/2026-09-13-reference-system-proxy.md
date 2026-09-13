# 参考图原生下载跟随系统代理

状态：代码修复与自动回归完成，待双平台安装包和 GUI 下载验收。用户报告预览版下载提示词时“参考图连接失败”，Windows 打包同时继续。

## 原因与方案

1. 页面 WebView 能加载图片，原生下载失败。macOS 已开启 HTTP/HTTPS 127.0.0.1:7890 系统代理，终端另有 HTTPS_PROXY；桌面 reqwest 0.12 关闭 default-features，未启用 system-proxy，导致 GUI 启动环境无法读取系统代理。cargo feature tree 确认缺失；清除终端代理变量后既有真实参考图测试失败。
2. 仅显式启用 reqwest 的 system-proxy，沿用手动代理优先、空值跟随系统的既有合同。保留域名白名单、禁重定向/会话、图片限额和原子导入，不用正文-only 静默绕过下载失败。
3. 在相同系统代理、移除所有终端代理变量的条件下重跑真实 HTTPS 下载至临时 SQLite，并核对附件字节；运行原生回归及发行检查。重新构建 GUI App，实际验证参考图下载。
4. 已公开 beta.1 标签不移动、不替换旧二进制。升级到 0.1.0-beta.2，为 macOS 与 Windows 构建同一修复提交的预览包。Windows 已运行的 beta.1 仅作构建基础验证，不公开该 Windows 资产；修复版仍验安装、启动、卸载后发布。

## Given / When / Then

- GIVEN GUI 无 HTTP_PROXY/HTTPS_PROXY 且 macOS/Windows 有系统 HTTP(S) 代理、应用手动代理为空；WHEN 原生下载参考图；THEN 使用系统代理，正文及校验后的图片可原子保存本地。
- GIVEN 手动代理已设置；WHEN 发原生请求；THEN 手动配置优先，非法地址拒绝保存。
- GIVEN 来源失效或图片超限；WHEN 下载；THEN 保留明确失败与完整导入边界，不伪称图片已保存。

## 验收

- 禁用所有终端代理变量的真实参考图测试，修复前以“参考图连接失败”失败；依赖树确认不含 system-proxy。
- 显式启用 reqwest/system-proxy，锁文件新增 macOS system-configuration 与 Windows registry 支持，未改变证书校验、重定向策略或附件导入事务。
- 修复首次复验仍遭一次连接失败，临时测试诊断后的两次相同条件请求均成功；诊断代码已移除。最终真实图下载至临时 SQLite、读取字节一致的测试通过（1.07 秒），不会写入用户库。源码中无诊断日志或额外凭据。
- 前端 497 项、原生 85 项通过（2 项显式环境测试默认忽略，真实图下载已单独运行）；发行检查 5 项通过。版本同步为 0.1.0-beta.2，待 GUI 及发布验证。
