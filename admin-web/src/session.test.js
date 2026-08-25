import { beforeEach, describe, expect, it } from "vitest";
import { loginAdmin, loginAdminOAuth, resetAdminSession, setAdminTransport } from "./session.js";

describe("admin session", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    resetAdminSession();
  });

  it("does not persist refresh in web storage", async () => {
    localStorage.setItem("refresh_token", "leaked");
    setAdminTransport(async () => ({
      access_token: "acc.admin",
      refresh_token: "ref.admin",
      email: "admin@promptark.local",
    }));
    const session = await loginAdmin({
      email: "admin@promptark.local",
      password: "adminpass",
    });
    expect(session.accessToken).toBe("acc.admin");
    expect(localStorage.getItem("refresh_token")).toBeNull();
    expect(sessionStorage.getItem("refresh_token")).toBeNull();
    expect(JSON.stringify(session)).not.toContain("ref.");
    expect(Object.keys(localStorage).some((key) => key.toLowerCase().includes("refresh"))).toBe(
      false,
    );
  });

  it("does not persist refresh in web storage after oauth", async () => {
    localStorage.setItem("refresh_token", "leaked");
    setAdminTransport(async () => ({
      access_token: "acc.oauth",
      refresh_token: "ref.oauth",
      email: "oauth@promptark.local",
    }));
    const session = await loginAdminOAuth("google");
    expect(session.accessToken).toBe("acc.oauth");
    expect(localStorage.getItem("refresh_token")).toBeNull();
    expect(sessionStorage.getItem("refresh_token")).toBeNull();
    expect(JSON.stringify(session)).not.toContain("ref.");
  });
});
