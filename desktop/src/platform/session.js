let accessToken = null;
let accountEmail = null;
let testTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function tauriInvoke(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}

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

export function resetMemorySession() {
  accessToken = null;
  accountEmail = null;
  testTransport = null;
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
}

export function setSessionTransport(transport) {
  testTransport = transport;
}

export function getSession() {
  return {
    email: accountEmail,
    accessToken,
    loggedIn: Boolean(accessToken),
  };
}

export async function loginSession({ email, password } = {}) {
  const title = String(email ?? "").trim();
  if (!title || !password) throw new Error("邮箱和密码不能为空");
  let result;
  if (testTransport) {
    result = await testTransport({ email: title, password });
  } else if (isTauri()) {
    result = await tauriInvoke("login_local_session", { email: title, password });
  } else {
    throw new Error("浏览器预览不持久化登录令牌");
  }
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
  accessToken = result.access_token ?? result.accessToken ?? null;
  accountEmail = result.email ?? title;
  return { email: accountEmail, accessToken };
}

export async function refreshSession() {
  let result;
  if (testTransport) {
    result = await testTransport({ refresh: true });
  } else if (isTauri()) {
    result = await tauriInvoke("refresh_local_session");
  } else {
    throw new Error("浏览器预览不持久化登录令牌");
  }
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
  accessToken = result.access_token ?? result.accessToken ?? null;
  if (result.email) accountEmail = result.email;
  return { email: accountEmail, accessToken };
}

export async function logoutSession() {
  const token = accessToken;
  accessToken = null;
  accountEmail = null;
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
  if (token && isTauri()) {
    try {
      await tauriInvoke("logout_local_session", { accessToken: token });
    } catch {
      /* 本地已清会话 */
    }
  }
}
