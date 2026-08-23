let accessToken = null;
let accountEmail = null;
let testTransport = null;

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
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
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
  if (typeof localStorage !== "undefined") stripRefreshFromWebStorage();
  accessToken = result.access_token ?? result.accessToken ?? null;
  accountEmail = result.email ?? title;
  return { email: accountEmail, accessToken };
}
