import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it } from "vitest";
import AdminApp from "./AdminApp.vue";
import { resetAdminSession, setAdminTransport, setOAuthProviderList } from "./session.js";
import { resetAdminApi, setAdminApiTransport } from "./adminApi.js";

describe("AdminApp", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    resetAdminSession();
    resetAdminApi();
    setOAuthProviderList([]);
    setAdminTransport(async () => ({
      access_token: "acc.admin",
      refresh_token: "ref.admin",
      email: "admin@promptark.local",
    }));
    setAdminApiTransport(async (request) => {
      if (request.kind === "list") {
        return { items: [{ id: "pub.1", source_id: "mem-1", status: "pending" }] };
      }
      if (request.kind === "approve") {
        return { id: request.id, source_id: "mem-1", status: "approved" };
      }
      if (request.kind === "users") {
        return {
          items: [
            { email: "admin@promptark.local", role: "admin" },
            { email: "dev@promptark.local", role: "user" },
          ],
        };
      }
      if (request.kind === "getSettings") {
        return { square_public: true };
      }
      if (request.kind === "putSettings") {
        return { square_public: request.square_public };
      }
      throw new Error(`unexpected ${request.kind}`);
    });
  });

  it("lists pending publications after admin login", async () => {
    const w = mount(AdminApp);
    await w.get('[data-testid="admin-email"]').setValue("admin@promptark.local");
    await w.get('[data-testid="admin-password"]').setValue("adminpass");
    await w.get('[data-testid="admin-login"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="review-list"]').text()).toContain("mem-1");
    expect(w.get('[data-testid="review-list"]').text()).toContain("pending");
  });

  it("approves a pending publication from the list", async () => {
    const w = mount(AdminApp);
    await w.get('[data-testid="admin-email"]').setValue("admin@promptark.local");
    await w.get('[data-testid="admin-password"]').setValue("adminpass");
    await w.get('[data-testid="admin-login"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="review-approve"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="review-list"]').text()).not.toContain("mem-1");
  });

  it("lists user emails and roles without password or delete controls", async () => {
    const w = mount(AdminApp);
    await w.get('[data-testid="admin-email"]').setValue("admin@promptark.local");
    await w.get('[data-testid="admin-password"]').setValue("adminpass");
    await w.get('[data-testid="admin-login"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="nav-users"]').trigger("click");
    await flushPromises();
    const list = w.get('[data-testid="user-list"]');
    expect(list.text()).toContain("admin@promptark.local");
    expect(list.text()).toContain("admin");
    expect(list.text()).toContain("dev@promptark.local");
    expect(list.text()).toContain("user");
    expect(w.find('[data-testid="user-password"]').exists()).toBe(false);
    expect(w.find('[data-testid="user-delete"]').exists()).toBe(false);
    expect(list.text()).not.toMatch(/改密|删除/);
  });

  it("saves the anonymous square setting", async () => {
    const calls = [];
    setAdminApiTransport(async (request) => {
      calls.push(request);
      if (request.kind === "list") {
        return { items: [] };
      }
      if (request.kind === "getSettings") {
        return { square_public: true };
      }
      if (request.kind === "putSettings") {
        return { square_public: request.square_public };
      }
      throw new Error(`unexpected ${request.kind}`);
    });
    const w = mount(AdminApp);
    await w.get('[data-testid="admin-email"]').setValue("admin@promptark.local");
    await w.get('[data-testid="admin-password"]').setValue("adminpass");
    await w.get('[data-testid="admin-login"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="nav-settings"]').trigger("click");
    await flushPromises();
    const box = w.get('[data-testid="setting-square-public"]');
    expect(box.element.checked).toBe(true);
    await box.setValue(false);
    await w.get('[data-testid="settings-save"]').trigger("click");
    await flushPromises();
    expect(calls.some((call) => call.kind === "putSettings" && call.square_public === false)).toBe(
      true,
    );
  });

  it("shows google on login when providers include google", async () => {
    setOAuthProviderList(["google"]);
    const w = mount(AdminApp);
    await flushPromises();
    expect(w.get('[data-testid="oauth-google"]').text()).toContain("Google");
    expect(w.find('[data-testid="oauth-github"]').exists()).toBe(false);
    expect(w.get('[data-testid="admin-email"]').exists()).toBe(true);
    expect(w.get('[data-testid="admin-password"]').exists()).toBe(true);
    expect(w.text()).not.toMatch(/QQ|LinuxDo/);
  });
});
