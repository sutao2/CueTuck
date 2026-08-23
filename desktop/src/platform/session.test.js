import { beforeEach, describe, expect, it } from "vitest";
import { loginSession, resetMemorySession, setSessionTransport } from "./session.js";

describe("session tokens", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    resetMemorySession();
  });

  it("does not persist refresh in web storage", async () => {
    localStorage.setItem("refresh_token", "leaked");
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    const session = await loginSession({ email: "dev@promptark.local", password: "devpass" });
    expect(session.accessToken).toBe("acc.1");
    expect(localStorage.getItem("refresh_token")).toBeNull();
    expect(sessionStorage.getItem("refresh_token")).toBeNull();
    expect(Object.keys(localStorage).some((key) => key.toLowerCase().includes("refresh"))).toBe(false);
  });
});
