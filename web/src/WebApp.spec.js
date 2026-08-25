import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { listLocalPrompts, resetMemoryLibrary } from "./memoryLibrary.js";
import { resetSquare, setSquareContentTransport, setSquareTransport, setFavoriteTransport } from "./square.js";
import {
  loginOAuthSession,
  loginSession,
  resetMemorySession,
  setOAuthProviderList,
  setSessionTransport,
} from "./session.js";
import WebApp from "./WebApp.vue";

describe("WebApp", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    resetMemoryLibrary();
    resetMemorySession();
    resetSquare();
    setOAuthProviderList([]);
  });

  it("keeps local space when the sidebar is collapsed", async () => {
    const w = mount(WebApp);
    expect(w.get('[data-space="local"]').exists()).toBe(true);
    expect(w.get('[data-testid="sidebar"]').classes()).not.toContain("is-collapsed");
    await w.get('[data-testid="toggle-sidebar"]').trigger("click");
    expect(w.get('[data-testid="sidebar"]').classes()).toContain("is-collapsed");
    expect(w.get('[data-space="local"]').exists()).toBe(true);
    expect(w.get('[data-region="content"]').exists()).toBe(true);
  });

  it("does not claim the browser library is synced to desktop sqlite", () => {
    const w = mount(WebApp);
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
    expect(w.text()).not.toContain("已与桌面库同步");
  });

  it("creates a memory prompt and lists it without claiming desktop sync", async () => {
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("测试");
    await w.get('[data-testid="prompt-content"]').setValue("正文");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    expect(w.get('[data-testid="prompt-list"]').text()).toContain("测试");
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
    expect(w.text()).not.toContain("已与桌面库同步");
  });

  it("opens a memory prompt and shows its body", async () => {
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("测试");
    await w.get('[data-testid="prompt-content"]').setValue("你好");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-row"]').trigger("click");
    expect(w.get('[data-testid="prompt-body"]').text()).toContain("你好");
  });

  it("updates a memory prompt title in the list after edit", async () => {
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("测试");
    await w.get('[data-testid="prompt-content"]').setValue("正文");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-row"]').trigger("click");
    await w.get('[data-testid="edit-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("已改");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    expect(w.get('[data-testid="prompt-list"]').text()).toContain("已改");
    expect(w.get('[data-testid="prompt-list"]').text()).not.toContain("测试");
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
  });

  it("fills wizard variables one at a time then previews and copies", async () => {
    const writeText = vi.fn();
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("行程");
    await w.get('[data-testid="prompt-content"]').setValue("去{{城市}}玩{{天数}}天");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    await w.get('[data-testid="use-prompt"]').trigger("click");
    expect(w.get('[data-testid="wizard-step"]').text()).toBe("城市");
    expect(w.text()).not.toContain("天数");
    await w.get('[data-testid="wizard-var"]').setValue("上海");
    await w.get('[data-testid="wizard-next"]').trigger("click");
    expect(w.get('[data-testid="wizard-step"]').text()).toBe("天数");
    await w.get('[data-testid="wizard-var"]').setValue("3");
    await w.get('[data-testid="wizard-next"]').trigger("click");
    expect(w.get('[data-testid="wizard-preview"]').text()).toBe("去上海玩3天");
    await w.get('[data-testid="wizard-copy"]').trigger("click");
    expect(writeText).toHaveBeenCalledWith("去上海玩3天");
  });

  it("skips fill and previews when there are no variables", async () => {
    const w = mount(WebApp);
    await w.get('[data-testid="new-prompt"]').trigger("click");
    await w.get('[data-testid="prompt-title"]').setValue("直出");
    await w.get('[data-testid="prompt-content"]').setValue("你好");
    await w.get('[data-testid="save-prompt"]').trigger("click");
    await w.get('[data-testid="use-prompt"]').trigger("click");
    expect(w.find('[data-testid="wizard-var"]').exists()).toBe(false);
    expect(w.get('[data-testid="wizard-preview"]').text()).toBe("你好");
  });

  it("shows a square offline notice and can return to local", async () => {
    setSquareTransport(async () => {
      throw new Error("广场暂时不可用");
    });
    const w = mount(WebApp);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="square-offline"]').exists()).toBe(true);
    await w.get('[data-testid="go-local"]').trigger("click");
    expect(w.get('[data-space="local"]').classes()).toContain("active");
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
  });

  it("downloads a square prompt into the memory library without claiming sqlite", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSquareContentTransport(async () => ({
      id: "sq-1",
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const w = mount(WebApp);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="square-list"]').text()).toContain("自然光群像");
    await w.get('[data-testid="square-favorite"]').trigger("click");
    expect(w.get('[data-testid="favorite-note"]').text()).toContain("收藏需要登录");
    expect(w.find('[data-testid="prompt-list"]').exists()).toBe(false);
    await w.get('[data-testid="square-download"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="prompt-list"]').text()).toContain("自然光群像");
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
    expect(w.text()).not.toContain("已写入本机 SQLite");
  });

  it("does not add a memory copy when favoriting while signed out", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const w = mount(WebApp);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    expect(listLocalPrompts()).toHaveLength(0);
    await w.get('[data-testid="square-favorite"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="favorite-note"]').text()).toContain("收藏需要登录");
    expect(listLocalPrompts()).toHaveLength(0);
  });

  it("favorites on the server without adding a memory copy", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSessionTransport(async () => ({
      access_token: "acc.web",
      refresh_token: "ref.web",
      email: "dev@promptark.local",
    }));
    const favoriteCalls = [];
    setFavoriteTransport(async (request) => {
      favoriteCalls.push(request);
      return { id: request.id };
    });
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WebApp);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="square-favorite"]').trigger("click");
    await flushPromises();
    expect(favoriteCalls.some((call) => call.method === "PUT" && call.id === "sq-1")).toBe(true);
    expect(listLocalPrompts()).toHaveLength(0);
  });

  it("does not write refresh to web storage after oauth", async () => {
    localStorage.setItem("refresh_token", "leaked");
    setSessionTransport(async () => ({
      access_token: "acc.oauth",
      refresh_token: "ref.oauth",
      email: "oauth@promptark.local",
    }));
    const session = await loginOAuthSession("google");
    expect(session.accessToken).toBe("acc.oauth");
    expect(localStorage.getItem("refresh_token")).toBeNull();
    expect(sessionStorage.getItem("refresh_token")).toBeNull();
    expect(JSON.stringify(session)).not.toContain("ref.");
  });

  it("shows google on login when providers include google", async () => {
    setOAuthProviderList(["google"]);
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const w = mount(WebApp);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="square-favorite"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="oauth-google"]').text()).toContain("Google");
    expect(w.find('[data-testid="oauth-github"]').exists()).toBe(false);
    expect(w.get('[data-testid="login-email"]').exists()).toBe(true);
    expect(w.text()).not.toMatch(/QQ|LinuxDo/);
  });
});
