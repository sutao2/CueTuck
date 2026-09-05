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
import { resetAccountLibrary, setAccountLibraryTransport } from "./accountLibrary.js";
import { resetBilling, setBillingTransport } from "./billing.js";
import WebApp from "./WebApp.vue";

describe("WebApp", () => {
  beforeEach(() => {
    localStorage.clear();
    sessionStorage.clear();
    resetMemoryLibrary();
    resetMemorySession();
    resetAccountLibrary();
    resetBilling();
    setBillingTransport({
      status: async () => ({ pro: false, payment_enabled: false, note: "支付未开通" }),
      checkout: async () => ({
        pro: false,
        payment_enabled: false,
        note: "支付未开通",
        checkout_url: null,
      }),
    });
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

  it("opens a downloaded collection member and retains failed downloads in square", async () => {
    setSquareTransport(async () => [{ id: "c", kind: "collection", title: "合集" }]);
    setSquareContentTransport(async () => ({ id: "c", kind: "collection", title: "合集", members: [] }));
    const w = mount(WebApp);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="square-download"]').trigger("click");
    await flushPromises();
    expect(w.text()).toContain("下载失败");
    setSquareContentTransport(async () => ({ id: "c", kind: "collection", title: "合集", members: [{ title: "成员", content: "可用正文" }] }));
    await w.get('[data-testid="square-download"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="collection-list"]').text()).toContain("合集");
    await w.get('[data-testid="collection-member"]').trigger("click");
    expect(w.get('[data-testid="prompt-body"]').text()).toBe("可用正文");
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

  it("shows the account library title after login without claiming sqlite", async () => {
    setAccountLibraryTransport({
      get: async () => ({
        items: [
          {
            id: "p-1",
            kind: "prompt",
            payload: { title: "本地仍在", content: "远端正文" },
            updated_at: "2",
          },
        ],
      }),
      put: async (items) => ({ items }),
    });
    setSessionTransport(async () => ({
      access_token: "acc.web",
      refresh_token: "ref.web",
      email: "dev@promptark.local",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WebApp);
    await flushPromises();
    expect(w.get('[data-testid="prompt-list"]').text()).toContain("本地仍在");
    expect(w.get('[data-testid="library-note"]').text()).toContain("尚未与桌面");
    expect(w.text()).not.toContain("已写入本机 SQLite");
  });

  it("shows unpaid billing as 支付未开通 and does not open checkout", async () => {
    const opened = [];
    vi.stubGlobal("open", (url) => {
      opened.push(url);
      return null;
    });
    setAccountLibraryTransport({
      get: async () => ({ items: [] }),
      put: async (items) => ({ items }),
    });
    setSessionTransport(async () => ({
      access_token: "acc.web",
      refresh_token: "ref.web",
      email: "dev@promptark.local",
    }));
    setBillingTransport({
      status: async () => ({ pro: false, payment_enabled: false, note: "支付未开通" }),
      checkout: async () => ({
        pro: false,
        payment_enabled: false,
        note: "支付未开通",
        checkout_url: null,
      }),
    });
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WebApp);
    await flushPromises();
    expect(w.get('[data-testid="billing-note"]').text()).toContain("支付未开通");
    expect(w.get('[data-testid="billing-pro"]').text()).toContain("未订阅");
    await w.get('[data-testid="billing-checkout"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="billing-note"]').text()).toContain("支付未开通");
    expect(opened).toEqual([]);
    expect(w.text()).not.toMatch(/已从商店|已经上架/);
    vi.unstubAllGlobals();
  });

  it("opens Stripe checkout only when a test checkout url is returned", async () => {
    const opened = [];
    vi.stubGlobal("open", (url) => {
      opened.push(url);
      return null;
    });
    setAccountLibraryTransport({
      get: async () => ({ items: [] }),
      put: async (items) => ({ items }),
    });
    setSessionTransport(async () => ({
      access_token: "acc.web",
      refresh_token: "ref.web",
      email: "dev@promptark.local",
    }));
    setBillingTransport({
      status: async () => ({ pro: false, payment_enabled: true, note: "" }),
      checkout: async () => ({
        pro: false,
        payment_enabled: true,
        note: "",
        checkout_url: "https://checkout.stripe.com/c/pay/cs_test_preview",
      }),
    });
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WebApp);
    await flushPromises();
    await w.get('[data-testid="billing-checkout"]').trigger("click");
    await flushPromises();
    expect(opened).toEqual(["https://checkout.stripe.com/c/pay/cs_test_preview"]);
    expect(w.get('[data-testid="billing-pro"]').text()).toContain("未订阅");
    expect(w.text()).not.toMatch(/已从商店|已经上架/);
    vi.unstubAllGlobals();
  });

  it("redeems a code without claiming a store listing", async () => {
    setAccountLibraryTransport({
      get: async () => ({ items: [] }),
      put: async (items) => ({ items }),
    });
    setSessionTransport(async () => ({
      access_token: "acc.web",
      refresh_token: "ref.web",
      email: "dev@promptark.local",
    }));
    setBillingTransport({
      status: async () => ({ pro: false, payment_enabled: false, note: "支付未开通" }),
      redeem: async (code) => {
        expect(code).toBe("PREVIEW-1");
        return { pro: true, payment_enabled: false, note: "" };
      },
    });
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WebApp);
    await flushPromises();
    await w.get('[data-testid="billing-redeem-code"]').setValue("PREVIEW-1");
    await w.get('[data-testid="billing-redeem"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="billing-pro"]').text()).toBe("Pro");
    expect(w.text()).not.toMatch(/已从商店|已经上架/);
  });
});
