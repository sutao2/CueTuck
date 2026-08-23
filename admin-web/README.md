# PromptArk 管理台

独立浏览器应用，不进桌面安装包。合同见 [admin.yaml](../docs/reference/openapi/admin.yaml)。本仓库 `backend/` 是预发。

```bash
cd admin-web
npm install
npm test
npm run dev
```

默认 `http://localhost:5174`。先另开终端跑 `backend`（`127.0.0.1:8787`）。管理员 `admin@promptark.local` / `adminpass`。Refresh 不写入 Web Storage。
