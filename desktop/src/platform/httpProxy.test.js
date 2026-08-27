import { describe, expect, it } from "vitest";
import { parseHttpProxy } from "./httpProxy.js";

describe("http proxy setting", () => {
  it("treats a blank value as follow-system", () => {
    expect(parseHttpProxy("")).toEqual({ ok: true, value: "" });
    expect(parseHttpProxy("  ")).toEqual({ ok: true, value: "" });
  });

  it("accepts http and https proxy urls", () => {
    expect(parseHttpProxy("http://127.0.0.1:7890")).toEqual({
      ok: true,
      value: "http://127.0.0.1:7890",
    });
    expect(parseHttpProxy("https://proxy.example:8443")).toEqual({
      ok: true,
      value: "https://proxy.example:8443",
    });
  });

  it("rejects socks, other schemes, and garbage", () => {
    expect(parseHttpProxy("socks5://127.0.0.1:1080")).toEqual({
      ok: false,
      error: "代理只支持 http 或 https",
    });
    expect(parseHttpProxy("ftp://127.0.0.1:21").ok).toBe(false);
    expect(parseHttpProxy("not-a-url")).toEqual({ ok: false, error: "代理地址无效" });
  });
});
