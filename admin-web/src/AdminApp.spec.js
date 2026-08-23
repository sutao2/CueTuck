import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it } from "vitest";
import AdminApp from "./AdminApp.vue";
import { resetAdminSession, setAdminTransport } from "./session.js";
import { resetAdminApi, setAdminApiTransport } from "./adminApi.js";

describe("AdminApp", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    resetAdminSession();
    resetAdminApi();
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
});
