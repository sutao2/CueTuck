# PromptArk 管理台

独立浏览器应用，不进桌面安装包。合同见 [admin.yaml](../docs/reference/openapi/admin.yaml)。本仓库 `backend/` 是预发。

```bash
cd admin-web
npm install
npm test
npm run dev
```

默认 `http://localhost:5174`。先另开终端跑 `backend`（`127.0.0.1:8787`）。管理员 `admin@promptark.local` / `adminpass`。Refresh 不写入 Web Storage。

## 配置第三方登录

登录后打开「第三方登录」，为 Google / GitHub 填写 Client ID、Client Secret 与回调地址，打开启用开关并保存。回调是后端的 `/v1/session/oauth/callback`，本机默认 `http://localhost:8787/v1/session/oauth/callback`，不是管理台地址；正式环境须使用 HTTPS。回调必须与提供商控制台登记值完全一致。

提供商侧创建应用的说明：[Google Web 应用 OAuth](https://developers.google.com/identity/protocols/oauth2/web-server)、[GitHub OAuth App](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/creating-an-oauth-app)。软件不会替你创建这些第三方应用，也不会把参数保存当作真实登录验收。

配置保存在 Postgres，立即影响后续登录请求。密钥输入留空保留原值，输入新值替换；停用保留凭据。数据库配置覆盖环境配置。普通第三方用户不会自动取得管理员身份。

后端首次启动自动生成 `.promptark/oauth.key`（相对后端工作目录），Unix 文件权限 0600。可通过 `PROMPTARK_OAUTH_KEY_FILE` 固定绝对路径。Client Secret 使用 AES-256-GCM 加密后入库，API 不回显原文或密文。数据库备份必须配套保管这个密钥文件，多实例共用同一密钥；密钥丢失时应恢复原文件，不能重新生成后覆盖已有配置。文件被 Git 忽略，不要提交或分享。未显式设置 OAuth state 签名密钥时，运行时从此独立密钥派生，避免使用公开的开发签名值。
