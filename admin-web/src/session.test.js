import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getAdminSession, loginAdmin, loginAdminOAuth, logoutAdmin, resetAdminSession, setAdminTransport } from "./session.js";

afterEach(() => vi.unstubAllGlobals());

it('does not restore an OAuth session after cancellation', async () => {
  resetAdminSession();
  let finish;
  setAdminTransport(() => new Promise(resolve => { finish = resolve; }));
  const controller = new AbortController();
  const pending = loginAdminOAuth('google', { signal: controller.signal });
  controller.abort(); finish({ access_token: 'late-token', email: 'admin@example.com' });
  await expect(pending).rejects.toThrow('取消');
  expect(getAdminSession().loggedIn).toBe(false);
});

it('rejects a regular login before storing a management session', async () => {
  resetAdminSession();
  vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce({ ok: true, json: async () => ({ access_token: 'user-token', email: 'user@example.com' }) }).mockResolvedValueOnce({ ok: false, status: 403 }));
  await expect(loginAdmin({ email: 'user@example.com', password: 'pass' })).rejects.toThrow('不是管理员');
  expect(getAdminSession().loggedIn).toBe(false);
});

it('verifies the administrator endpoint and revokes the session when leaving', async () => {
  resetAdminSession();
  const fetch = vi.fn().mockResolvedValueOnce({ ok: true, json: async () => ({ access_token: 'admin-token', refresh_token: 'private-refresh' }) })
    .mockResolvedValueOnce({ ok: true, json: async () => ({ email: 'admin@example.com', role: 'admin' }) }).mockResolvedValueOnce({ ok: true });
  vi.stubGlobal('fetch', fetch);
  await loginAdmin({ email: 'admin@example.com', password: 'pass' });
  expect(fetch.mock.calls[1][0]).toContain('/v1/admin/me');
  expect(getAdminSession().email).toBe('admin@example.com');
  expect(await logoutAdmin()).toBe(true);
  expect(getAdminSession().loggedIn).toBe(false);
  expect(fetch.mock.calls[2][1].method).toBe('DELETE');
});

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
