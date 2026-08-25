import { getSession } from "./session.js";

let testTransport = null;

function isTauri() {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

async function tauriInvoke(command, args) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(command, args);
}

function apiBase() {
  return "http://127.0.0.1:8787";
}

function requireAccessToken() {
  const token = getSession().accessToken;
  if (!token) throw new Error("账单需要登录");
  return token;
}

export function resetBilling() {
  testTransport = null;
}

export function setBillingTransport(transport) {
  testTransport = transport;
}

export async function getBillingStatus() {
  if (testTransport?.status) return testTransport.status();
  const token = requireAccessToken();
  if (isTauri()) {
    return tauriInvoke("get_billing_status", { access_token: token });
  }
  const response = await fetch(`${apiBase()}/v1/billing/status`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) throw new Error("账单暂时不可用");
  return response.json();
}

export async function startBillingCheckout() {
  if (testTransport?.checkout) return testTransport.checkout();
  const token = requireAccessToken();
  if (isTauri()) {
    return tauriInvoke("start_billing_checkout", { access_token: token });
  }
  const response = await fetch(`${apiBase()}/v1/billing/checkout`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
  });
  const payload = await response.json().catch(() => ({}));
  if (response.status === 401) throw new Error("账单需要登录");
  return payload;
}

export async function redeemBillingCode(code) {
  if (testTransport?.redeem) return testTransport.redeem(code);
  const token = requireAccessToken();
  const trimmed = String(code ?? "").trim();
  if (!trimmed) throw new Error("兑换码不能为空");
  if (isTauri()) {
    return tauriInvoke("redeem_billing_code", { access_token: token, code: trimmed });
  }
  const response = await fetch(`${apiBase()}/v1/billing/redeem`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ code: trimmed }),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(payload.note || "兑换失败");
  }
  return payload;
}
