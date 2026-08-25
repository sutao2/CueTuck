let accessToken = null;
let accountEmail = null;
let testTransport = null;
let testMeTransport = null;
let oauthProviderList = [];

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
  testMeTransport = null;
  oauthProviderList = [];
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
}

export function setOAuthProviderList(items) {
  oauthProviderList = Array.isArray(items) ? [...items] : [];
}

export function setSessionTransport(transport) {
  testTransport = transport;
}

export function setMeTransport(transport) {
  testMeTransport = transport;
}

export function getSession() {
  return {
    email: accountEmail,
    accessToken,
    loggedIn: Boolean(accessToken),
  };
}

function applySession(result, fallbackEmail) {
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
  accessToken = result.access_token ?? result.accessToken ?? null;
  accountEmail = result.email ?? fallbackEmail ?? null;
  return { email: accountEmail, accessToken };
}

export async function listOAuthProviders() {
  if (isTauri() && !testTransport) {
    return tauriInvoke("list_oauth_providers");
  }
  return { items: oauthProviderList };
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
  return applySession(result, title);
}

export async function loginOAuthSession(provider, { signal } = {}) {
  const name = String(provider ?? "").trim().toLowerCase();
  if (name !== "google" && name !== "github") {
    throw new Error("不支持的登录方式");
  }
  let result;
  if (testTransport) {
    result = await testTransport({ provider: name });
  } else if (isTauri()) {
    const flowId = await tauriInvoke("start_oauth_session", { provider: name });
    const cancelFlow = () => tauriInvoke("cancel_oauth_session", { flowId }).catch(() => {});
    if (signal) {
      if (signal.aborted) {
        await cancelFlow();
        throw new Error("已取消");
      }
      signal.addEventListener("abort", () => {
        cancelFlow();
      }, { once: true });
    }
    const deadline = Date.now() + 180_000;
    while (Date.now() < deadline) {
      if (signal?.aborted) throw new Error("已取消");
      const ready = await tauriInvoke("poll_oauth_session", { flowId });
      if (signal?.aborted) throw new Error("已取消");
      if (ready) {
        result = await tauriInvoke("commit_oauth_session", { flowId });
        break;
      }
      await wait(400, signal);
    }
    if (!result) throw new Error("登录超时");
  } else {
    throw new Error("浏览器预览不持久化登录令牌");
  }
  return applySession(result, null);
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

export async function refreshSession() {
  let result;
  if (testTransport) {
    result = await testTransport({ refresh: true });
  } else if (isTauri()) {
    result = await tauriInvoke("refresh_local_session");
  } else {
    throw new Error("浏览器预览不持久化登录令牌");
  }
  return applySession(result, accountEmail);
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

export async function getMe() {
  if (testMeTransport?.get) return testMeTransport.get();
  const token = accessToken;
  if (!token) throw new Error("查看资料需要登录");
  if (isTauri()) {
    return tauriInvoke("get_me", { access_token: token });
  }
  throw new Error("查看资料需要登录");
}

export async function putMe({ displayName = "", bio = "" } = {}) {
  if (testMeTransport?.put) {
    return testMeTransport.put({ display_name: displayName, bio });
  }
  const token = accessToken;
  if (!token) throw new Error("保存资料需要登录");
  if (isTauri()) {
    return tauriInvoke("put_me", {
      access_token: token,
      display_name: displayName,
      bio,
    });
  }
  throw new Error("保存资料需要登录");
}
