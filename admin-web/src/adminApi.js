import { getAdminSession } from "./session.js";

const API_BASE = import.meta.env.VITE_API_BASE || "http://127.0.0.1:8787";

let testTransport = null;

export function setAdminApiTransport(transport) {
  testTransport = transport;
}

export function resetAdminApi() {
  testTransport = null;
}

async function request(kind, extra = {}) {
  if (testTransport) {
    return testTransport({ kind, ...extra });
  }
  const { accessToken } = getAdminSession();
  if (!accessToken) throw new Error("需要先登录");
  const path =
    kind === "list"
      ? "/v1/admin/publications"
      : kind === "users"
        ? "/v1/admin/users"
        : kind === "getSettings" || kind === "putSettings"
          ? "/v1/admin/settings"
          : `/v1/admin/publications/${extra.id}/${kind === "approve" ? "approve" : "reject"}`;
  const headers = { authorization: `Bearer ${accessToken}` };
  const init = { method: "GET", headers };
  if (kind === "putSettings") {
    init.method = "PUT";
    headers["content-type"] = "application/json";
    init.body = JSON.stringify({ square_public: extra.square_public });
  } else if (kind !== "list" && kind !== "users" && kind !== "getSettings") {
    init.method = "POST";
  }
  const response = await fetch(`${API_BASE}${path}`, init);
  if (response.status === 403) throw new Error("需要管理员账号");
  if (!response.ok) throw new Error("管理请求失败");
  return response.json();
}

export function listPendingPublications() {
  return request("list");
}

export function approvePublication(id) {
  return request("approve", { id });
}

export function rejectPublication(id) {
  return request("reject", { id });
}

export function listAdminUsers() {
  return request("users");
}

export function getAdminSettings() {
  return request("getSettings");
}

export function putAdminSettings(squarePublic) {
  return request("putSettings", { square_public: squarePublic });
}
