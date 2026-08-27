export function parseHttpProxy(raw) {
  const trimmed = String(raw ?? "").trim();
  if (!trimmed) return { ok: true, value: "" };
  if (!/^https?:\/\//i.test(trimmed)) {
    return {
      ok: false,
      error: trimmed.includes("://") ? "代理只支持 http 或 https" : "代理地址无效",
    };
  }
  let parsed;
  try {
    parsed = new URL(trimmed);
  } catch {
    return { ok: false, error: "代理地址无效" };
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return { ok: false, error: "代理只支持 http 或 https" };
  }
  if (!parsed.hostname) {
    return { ok: false, error: "代理地址无效" };
  }
  return { ok: true, value: trimmed };
}
