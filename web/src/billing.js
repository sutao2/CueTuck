import { getSession } from "./session.js";

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:8787";

let testTransport = null;
let pendingCheckout = null, pendingRedeem = null;

export function resetBilling() {
  testTransport = null;
  pendingCheckout = null; pendingRedeem = null;
}

export function setBillingTransport(transport) {
  testTransport = transport;
}

function requireAccessToken() {
  const token = getSession().accessToken;
  if (!token) throw new Error("账单需要登录");
  return token;
}

export async function getBillingStatus() {
  if (testTransport?.status) return testTransport.status();
  const token = requireAccessToken();
  const response = await fetch(`${API_BASE}/v1/billing/status`, {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!response.ok) throw new Error("账单暂时不可用");
  return response.json();
}

export async function startBillingCheckout(mockOutcome) {
  if (testTransport?.checkout) return testTransport.checkout(mockOutcome);
  const token = requireAccessToken();
  const key=`${token}:${mockOutcome}`;
  if(mockOutcome&&pendingCheckout?.key!==key)pendingCheckout={key,id:crypto.randomUUID()};
  const requestId=mockOutcome?pendingCheckout.id:undefined;
  const response = await fetch(`${API_BASE}/v1/billing/checkout`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
    body: JSON.stringify({ mock_outcome: mockOutcome, request_id:requestId }),
  });
  const payload = await response.json().catch(() => ({}));
  if (response.status === 401) throw new Error("账单需要登录");
  if (!response.ok && !([403, 409].includes(response.status) && typeof payload.note === "string")) throw new Error("支付请求失败，请重试");
  pendingCheckout=null;return payload;
}

export async function redeemBillingCode(code,{mock=false}={}) {
  if (testTransport?.redeem) return testTransport.redeem(code);
  const token = requireAccessToken();
  const trimmed = String(code ?? "").trim();
  if (!trimmed) throw new Error("兑换码不能为空");
  const key=`${token}:${mock}:${trimmed}`;if(mock&&pendingRedeem?.key!==key)pendingRedeem={key,id:crypto.randomUUID()};
  const response = await fetch(`${API_BASE}/v1/billing/${mock?'mock/':''}redeem`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ code: trimmed, ...(mock?{request_id:pendingRedeem.id}:{}) }),
  });
  const payload = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(payload.note || "兑换失败");
  }
  pendingRedeem=null;return payload;
}
