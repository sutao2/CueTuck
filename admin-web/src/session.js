let accessToken = null;
let accountEmail = null;
let testTransport = null;
let oauthProviderList = [];
let oauthProviderOverride = false;

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:8787";

function stripRefreshFromWebStorage() {
  for (const storage of [localStorage, sessionStorage]) {
    const keys = [];
    for (let index = 0; index < storage.length; index += 1) {
      const key = storage.key(index);
      if (key && key.toLowerCase().includes("refresh")) keys.push(key);
    }
    keys.forEach((key) => storage.removeItem(key));
  }
}

export function resetAdminSession() {
  accessToken = null;
  accountEmail = null;
  testTransport = null;
  oauthProviderList = [];
  oauthProviderOverride = false;
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
}

export function setOAuthProviderList(items) {
  oauthProviderList = Array.isArray(items) ? [...items] : [];
  oauthProviderOverride = true;
}

export function setAdminTransport(transport) {
  testTransport = transport;
}

export function getAdminSession() {
  return {
    email: accountEmail,
    accessToken,
    loggedIn: Boolean(accessToken),
  };
}

async function applySession(result, fallbackEmail, signal) {
  if (signal?.aborted) throw new Error('已取消');
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
  accessToken = null;
  accountEmail = null;
  const candidate = result.access_token ?? result.accessToken;
  if (!candidate) throw new Error('登录响应无效');
  let email = result.email ?? fallbackEmail;
  if (!testTransport) {
    const response = await fetch(`${API_BASE}/v1/admin/me`, { headers: { authorization: `Bearer ${candidate}` }, signal });
    if (!response.ok) throw new Error(response.status === 403 ? '该账号不是管理员' : '无法验证管理员身份，请重新登录');
    const account = await response.json();
    if (account.role !== 'admin') throw new Error('该账号不是管理员');
    email = account.email;
  }
  if (signal?.aborted) throw new Error('已取消');
  accessToken = candidate;
  accountEmail = email;
  return { email: accountEmail, accessToken };
}

export async function logoutAdmin() {
  const token = accessToken;
  accessToken = null;
  accountEmail = null;
  if (!token || testTransport) return true;
  try { return (await fetch(`${API_BASE}/v1/session`, { method: 'DELETE', headers: { authorization: `Bearer ${token}` } })).ok; }
  catch { return false; }
}

export async function listOAuthProviders() {
  if (testTransport || oauthProviderOverride) {
    return { items: oauthProviderList };
  }
  try {
    const response = await fetch(`${API_BASE}/v1/session/oauth/providers`);
    if (!response.ok) return { items: [] };
    return response.json();
  } catch {
    return { items: [] };
  }
}

export async function loginAdmin({ email, password } = {}) {
  const title = String(email ?? "").trim();
  if (!title || !password) throw new Error("邮箱和密码不能为空");
  let result;
  if (testTransport) {
    result = await testTransport({ email: title, password });
  } else {
    const response = await fetch(`${API_BASE}/v1/session`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ email: title, password }),
    });
    if (!response.ok) {
      throw new Error(response.status === 401 ? "邮箱或密码不对" : "登录失败");
    }
    result = await response.json();
  }
  return applySession(result, title);
}

export async function loginAdminOAuth(provider, { signal } = {}) {
  const name = String(provider ?? "").trim().toLowerCase();
  if (name !== "google" && name !== "github") {
    throw new Error("不支持的登录方式");
  }
  let result;
  if (testTransport) {
    result = await testTransport({ provider: name });
  } else {
    result = await pollBrowserOAuth(name, signal);
  }
  return applySession(result, null, signal);
}

async function pollBrowserOAuth(provider, signal) {
  const flowId = crypto.randomUUID();
  const start = `${API_BASE}/v1/session/oauth/${provider}?response_mode=browser&flow_id=${encodeURIComponent(flowId)}`;
  if (typeof window !== "undefined" && typeof window.open === "function") {
    window.open(start, "_blank", "noopener");
  }
  const deadline = Date.now() + 180_000;
  while (Date.now() < deadline) {
    if (signal?.aborted) throw new Error("已取消");
    try {
      const response = await fetch(
        `${API_BASE}/v1/session/oauth/session/${encodeURIComponent(flowId)}`,
      );
      if (response.ok) {
        const payload = await response.json();
        if (payload.status === "ready" && payload.session) {
          return payload.session;
        }
      }
    } catch {
      /* 轮询失败时继续等到截止 */
    }
    await wait(400, signal);
  }
  throw new Error("登录超时");
}

function wait(ms, signal) {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new Error("已取消"));
      return;
    }
    const timer = setTimeout(resolve, ms);
    if (!signal) return;
    signal.addEventListener(
      "abort",
      () => {
        clearTimeout(timer);
        reject(new Error("已取消"));
      },
      { once: true },
    );
  });
}
