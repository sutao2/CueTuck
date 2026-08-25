import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, it, expect, vi } from "vitest";
import WorkbenchShell from "./WorkbenchShell.vue";
import {
  createLocalCollection,
  createLocalPrompt,
  listLocalPrompts,
  resetMemoryLibrary,
  getLocalSetting,
  setLocalSetting,
  recordLocalPromptUse,
} from "../platform/library.js";
import {
  resetMemorySession,
  setSessionTransport,
  setOAuthProviderList,
  setMeTransport,
  loginSession,
  logoutSession,
} from "../platform/session.js";
import {
  resetLibrarySync,
  setLibrarySyncTransport,
} from "../platform/librarySync.js";
import { resetUpdates, setUpdateTransport } from "../platform/updates.js";
import {
  resetSquare,
  setFavoriteTransport,
  setMineTransport,
  setPublishTransport,
  setSquareContentTransport,
  setSquareTransport,
} from "../platform/square.js";

describe("WorkbenchShell", () => {
  beforeEach(() => {
    resetMemoryLibrary();
    resetMemorySession();
    resetLibrarySync();
    resetUpdates();
    resetSquare();
    setSquareTransport(async () => {
      throw new Error("广场暂时不可用");
    });
    setFavoriteTransport(async (request) => {
      if (request.method === "GET") return { items: [] };
      return { id: request.id };
    });
  });

  it("renders four chrome regions", () => {
    const w = mount(WorkbenchShell);
    expect(w.get('[data-region="titlebar"]').exists()).toBe(true);
    expect(w.get('[data-region="sidebar"]').exists()).toBe(true);
    expect(w.get('[data-region="content"]').exists()).toBe(true);
    expect(w.get('[data-region="statusbar"]').exists()).toBe(true);
  });

  it("shows a non-blocking offline notice and can return to local", async () => {
    await createLocalPrompt({ title: "本地仍在", content: "x" });
    setSquareTransport(async () => {
      throw new Error("广场暂时不可用");
    });
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="square-offline"]').text()).toContain("离线");
    expect(w.get('[data-testid="go-local"]').exists()).toBe(true);
    await w.get('[data-testid="go-local"]').trigger("click");
    await flushPromises();
    expect(w.text()).toContain("本地仍在");
    expect(w.find('[data-testid="square-offline"]').exists()).toBe(false);
  });

  it("shows square items in the content grid not the category tree", async () => {
    setSquareTransport(async () => [
      { id: "col-sq", title: "人像灵感合集", kind: "collection" },
      { id: "sq-1", title: "自然光群像", kind: "prompt" },
    ]);
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("人像灵感合集");
    expect(w.get('[data-testid="library-view"]').text()).toContain("自然光群像");
    expect(w.get(".category-tree").text()).not.toContain("人像灵感合集");
    expect(w.find('[data-testid="square-offline"]').exists()).toBe(false);
  });

  it("renders prototype sidebar chrome", () => {
    const w = mount(WorkbenchShell);
    expect(w.get(".space-tab .nav-icon").exists()).toBe(true);
    expect(w.get(".category-tree").exists()).toBe(true);
    expect(w.get(".statusbar .status-item").exists()).toBe(true);
  });

  it("opens create modal from primary action", async () => {
    const w = mount(WorkbenchShell);
    await w.get(".primary-button").trigger("click");
    expect(w.get('[data-testid="prompt-editor"]').exists()).toBe(true);
  });

  it("loads preset categories into the tree", async () => {
    const w = mount(WorkbenchShell);
    await flushPromises();
    expect(w.text()).toContain("软件开发");
    expect(w.text()).toContain("网站开发");
  });

  it("creates a collection in the content grid", async () => {
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get(".content-actions .primary-button").trigger("click");
    const types = w.findAll(".create-type");
    await types[1].trigger("click");
    await w.get(".create-body input").setValue("人像灵感");
    const coverSelect = w.findAll(".create-body select").at(1);
    await coverSelect.setValue("grid");
    expect(w.get('[data-testid="cover-files"]').exists()).toBe(true);
    await w.get(".modal-footer .primary-button").trigger("click");
    await flushPromises();
    expect(w.text()).toContain("人像灵感");
    expect(w.text()).toContain("合集");
  });

  it("does not shrink library count when filtering", async () => {
    const w = mount(WorkbenchShell);
    await createLocalPrompt({ title: "未分类", content: "x" });
    await flushPromises();
    await w.get(".tree-row").trigger("click");
    await flushPromises();
    expect(w.emitted("library-changed").at(-1)).toEqual([1]);
    const portrait = w.findAll(".tree-row.child").find((row) => row.text().includes("人像摄影"));
    await portrait.trigger("click");
    await flushPromises();
    expect(w.emitted("library-changed").at(-1)).toEqual([1]);
  });

  it("uses mac chrome on macos", () => {
    const w = mount(WorkbenchShell, { props: { host: "macos" } });
    expect(w.get('[data-region="titlebar"]').classes()).toContain("host-mac");
    expect(w.get('[data-region="titlebar"] kbd').text()).toBe("⌃Space");
    expect(w.find(".window-controls").exists()).toBe(false);
  });

  it("shows the same prompts as rows in list view", async () => {
    await createLocalPrompt({ title: "行视图A", content: "列表摘要" });
    const w = mount(WorkbenchShell);
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').attributes("data-layout")).toBe("grid");
    await w.get('[title="列表视图"]').trigger("click");
    expect(w.get('[data-testid="library-view"]').attributes("data-layout")).toBe("list");
    expect(w.get(".prompt-card").classes()).toContain("as-row");
    expect(w.get(".prompt-card").text()).toContain("行视图A");
  });

  it("shows the first three cover images on a collection card", async () => {
    await createLocalCollection({
      title: "人像灵感",
      coverType: "grid",
      coverUrls: ["one.jpg", "two.jpg", "three.jpg"],
    });
    const w = mount(WorkbenchShell);
    await flushPromises();
    const preview = w.get('[data-testid="collection-cover-preview"]');
    expect(preview.findAll("img")).toHaveLength(3);
    await w.get(".prompt-card.collection").trigger("click");
    await flushPromises();
    expect(w.findAll('[data-testid="cover-grid"] img')).toHaveLength(3);
    expect(w.findAll('[data-testid="cover-grid"] i')).toHaveLength(9);
  });

  it("adds a local child category under the selected parent", async () => {
    const w = mount(WorkbenchShell);
    await flushPromises();
    const office = w.findAll(".tree-parent").find((row) => row.text().includes("办公效率"));
    await office.trigger("click");
    await w.get('[data-testid="add-category"]').trigger("click");
    await w.get('[data-testid="new-category-name"]').setValue("周报");
    await w.get('[data-testid="confirm-category"]').trigger("click");
    await flushPromises();
    expect(w.text()).toContain("周报");
  });

  it("refuses a third-level category from a child", async () => {
    const w = mount(WorkbenchShell);
    await flushPromises();
    const frontend = w.findAll(".tree-row.child").find((row) => row.text().includes("前端工程"));
    await frontend.trigger("click");
    await w.get('[data-testid="add-category"]').trigger("click");
    expect(w.get('[data-testid="category-error"]').text()).toContain("小分类下不能再创建子分类");
    expect(w.find('[data-testid="new-category-name"]').exists()).toBe(false);
  });

  it("downloads a square prompt without login as source=downloaded", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="download-square"]').trigger("click");
    await flushPromises();
    expect(w.find('[data-testid="login-modal"]').exists()).toBe(false);
    const rows = await listLocalPrompts({ query: "自然光群像" });
    expect(rows).toHaveLength(1);
    expect(rows[0].source).toBe("downloaded");
    await w.get('[data-space="local"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("自然光群像");
  });

  it("keeps author on download when the setting is on", async () => {
    setSquareTransport(async () => [
      { id: "sq-keep", title: "自然光群像", kind: "prompt", author: "林晚" },
    ]);
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
      author: "林晚",
    }));
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await w.get('[data-testid="keep-author-on-download"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("keep_author_on_download")).toBe("1");
    await w.get(".modal-close").trigger("click");

    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="download-square"]').trigger("click");
    await flushPromises();
    const kept = await listLocalPrompts({ query: "自然光群像" });
    expect(kept[0].author).toBe("林晚");
    expect(kept[0].content).toBe("清透蓝天下的多元人物群像。");
    await w.get('[data-space="local"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("林晚");

    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await w.get('[data-testid="keep-author-on-download"]').setValue(false);
    await flushPromises();
    expect(await getLocalSetting("keep_author_on_download")).toBe("0");
    await w.get(".modal-close").trigger("click");

    setSquareTransport(async () => [
      { id: "sq-plain", title: "夜景街拍", kind: "prompt", author: "林晚" },
    ]);
    setSquareContentTransport(async (id) => ({
      id,
      title: "夜景街拍",
      content: "潮湿路面的霓虹倒影。",
      author: "林晚",
    }));
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="download-square"]').trigger("click");
    await flushPromises();
    const skipped = await listLocalPrompts({ query: "夜景街拍" });
    expect(skipped[0].author).toBeFalsy();
    expect(skipped[0].content).toBe("潮湿路面的霓虹倒影。");
    await w.get('[data-space="local"]').trigger("click");
    await flushPromises();
    const nightCard = w.findAll(".prompt-card").find((card) => card.text().includes("夜景街拍"));
    expect(nightCard.text()).not.toContain("林晚");
  });

  it("opens login from favorite without writing a local copy", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="favorite-square"]').trigger("click");
    expect(w.get('[data-testid="login-reason"]').text()).toContain("收藏");
    expect(await listLocalPrompts({ query: "" })).toHaveLength(0);
  });

  it("shows google on login when providers include google", async () => {
    setOAuthProviderList(["google"]);
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="favorite-square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="oauth-google"]').text()).toContain("Google");
    expect(w.find('[data-testid="oauth-github"]').exists()).toBe(false);
    expect(w.get('[data-testid="login-modal"]').text()).not.toMatch(/QQ|LinuxDo/);
  });

  it("disables oauth while a provider login is in flight", async () => {
    setOAuthProviderList(["google"]);
    let release;
    const pending = new Promise((resolve) => {
      release = resolve;
    });
    setSessionTransport(async (request) => {
      if (request.provider === "google") {
        await pending;
        return {
          access_token: "acc.oauth",
          refresh_token: "ref.oauth",
          email: "oauth@promptark.local",
        };
      }
      return {
        access_token: "acc.1",
        refresh_token: "ref.1",
        email: "dev@promptark.local",
      };
    });
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="favorite-square"]').trigger("click");
    await flushPromises();
    const click = w.get('[data-testid="oauth-google"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="oauth-google"]').element.disabled).toBe(true);
    expect(w.get('[data-testid="login-submit"]').element.disabled).toBe(true);
    expect(w.get('[data-testid="oauth-wait"]').exists()).toBe(true);
    release();
    await click;
    await flushPromises();
    expect(w.find('[data-testid="login-modal"]').exists()).toBe(false);
  });

  it("hides oauth buttons when providers empty", async () => {
    setOAuthProviderList([]);
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="favorite-square"]').trigger("click");
    await flushPromises();
    expect(w.find('[data-testid="oauth-google"]').exists()).toBe(false);
    expect(w.find('[data-testid="oauth-github"]').exists()).toBe(false);
    expect(w.get('[data-testid="login-email"]').exists()).toBe(true);
    expect(w.get('[data-testid="login-password"]').exists()).toBe(true);
  });

  it("favorites a square item while logged in without writing a local copy", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    const favoriteCalls = [];
    setFavoriteTransport(async (request) => {
      favoriteCalls.push(request);
      if (request.method === "GET") return { items: [] };
      return { id: request.id };
    });
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="favorite-square"]').trigger("click");
    await flushPromises();
    expect(favoriteCalls.some((call) => call.method === "PUT" && call.id === "sq-1")).toBe(true);
    expect(w.find('[data-testid="login-reason"]').exists()).toBe(false);
    expect(await listLocalPrompts({ query: "" })).toHaveLength(0);
  });

  it("keeps a downloaded copy after unfavorite", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    const ids = new Set(["sq-1"]);
    setFavoriteTransport(async (request) => {
      if (request.method === "GET") return { items: [...ids].map((id) => ({ id })) };
      if (request.method === "DELETE") ids.delete(request.id);
      return { id: request.id };
    });
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="download-square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="favorite-square"]').text()).toBe("已收藏");
    await w.get('[data-testid="favorite-square"]').trigger("click");
    await flushPromises();
    const rows = await listLocalPrompts({ query: "自然光群像" });
    expect(rows).toHaveLength(1);
    expect(rows[0].source).toBe("downloaded");
  });

  it("opens login from publish and resumes after success", async () => {
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await w.get('[data-testid="publish-prompt"]').trigger("click");
    expect(w.get('[data-testid="login-reason"]').text()).toContain("发布需要登录");
    await w.get('[data-testid="login-email"]').setValue("dev@promptark.local");
    await w.get('[data-testid="login-password"]').setValue("devpass");
    await w.get('[data-testid="login-submit"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="publish-resume"]').exists()).toBe(true);
  });

  it("disables publish submit until a local source is selected", async () => {
    const created = await createLocalPrompt({ title: "本地源", content: "旧正文" });
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get('[data-space="square"]').trigger("click");
    await w.get('[data-testid="publish-prompt"]').trigger("click");
    await w.get('[data-testid="login-email"]').setValue("dev@promptark.local");
    await w.get('[data-testid="login-password"]').setValue("devpass");
    await w.get('[data-testid="login-submit"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="publish-submit"]').element.disabled).toBe(true);
    await w.get('[data-testid="publish-source"]').setValue(created.id);
    expect(w.get('[data-testid="publish-submit"]').element.disabled).toBe(false);
  });

  it("keeps the local prompt editable after publish", async () => {
    const created = await createLocalPrompt({ title: "本地源", content: "旧正文" });
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    setPublishTransport(async () => ({ id: "pub-1", status: "pending" }));
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get('[data-space="square"]').trigger("click");
    await w.get('[data-testid="publish-prompt"]').trigger("click");
    await w.get('[data-testid="login-email"]').setValue("dev@promptark.local");
    await w.get('[data-testid="login-password"]').setValue("devpass");
    await w.get('[data-testid="login-submit"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="publish-source"]').setValue(created.id);
    await w.get('[data-testid="publish-submit"]').trigger("click");
    await flushPromises();
    await w.get('[data-space="local"]').trigger("click");
    await flushPromises();
    await w.get(".prompt-card").trigger("click");
    expect(w.get('[data-testid="prompt-editor"] textarea').element.disabled).toBe(false);
    await w.get('[data-testid="prompt-editor"] textarea').setValue("新正文");
    await w.get(".modal-footer .primary-button").trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("新正文");
    const rows = await listLocalPrompts({ query: "本地源" });
    expect(rows[0].content).toBe("新正文");
  });

  it("opens settings from the sidebar", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    expect(w.get('[data-testid="settings-modal"]').exists()).toBe(true);
    await w.get('[data-settings-page="sync"]').trigger("click");
    expect(w.get('[data-testid="settings-unavailable"]').text()).toContain("启动器与 MCP 仍只读本机 SQLite");
    expect(w.get('[data-testid="settings-unavailable"]').text()).toContain("尚未提供");
  });

  it("lists ten settings categories", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    expect(w.findAll("[data-settings-page]").map((button) => button.text())).toEqual([
      "常规",
      "账号与广场",
      "快捷键",
      "同步",
      "AI 与模型",
      "数据与备份",
      "网络与代理",
      "外观",
      "隐私与安全",
      "更新",
    ]);
  });

  it("keeps the updates page without claiming a store check", async () => {
    setUpdateTransport(async () => ({ available: false, notes: "" }));
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="updates"]').trigger("click");
    const panel = w.get('[data-testid="settings-updates"]');
    expect(panel.text()).toContain("当前版本");
    expect(panel.text()).toContain("检查更新");
    expect(panel.text()).toContain("自动下载");
    expect(panel.text()).toContain("更新通道");
    expect(panel.text()).toContain("发行说明");
    await w.get('[data-testid="check-updates"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="update-note"]').text()).toContain("没有可用更新");
    expect(panel.text()).not.toMatch(/已从商店|已经连上更新服务器/);
  });

  it("does not treat a failed update check as no updates", async () => {
    setUpdateTransport(async () => {
      throw new Error("检查失败");
    });
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="updates"]').trigger("click");
    await w.get('[data-testid="check-updates"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="update-note"]').text()).toContain("检查失败");
    expect(w.get('[data-testid="update-note"]').text()).not.toContain("没有可用更新");
    expect(w.get('[data-testid="settings-updates"]').text()).not.toMatch(/已从商店|已经连上更新服务器/);
  });

  it("shows sync rows without requesting the backend", async () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="sync"]').trigger("click");
    const panel = w.get('[data-testid="settings-unavailable"]');
    expect(panel.text()).toContain("尚未提供");
    expect(panel.text()).toContain("自动同步收藏");
    expect(panel.text()).toContain("仅在 Wi-Fi");
    expect(panel.text()).toContain("冲突处理");
    expect(panel.text()).toContain("立即同步");
    await w.get('[data-testid="sync-now"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="login-modal"]').exists()).toBe(true);
    expect(w.find('[data-testid="sync-note"]').exists()).toBe(false);
    expect(fetchSpy).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });

  it("pushes the local library to the account when signed in and syncing now", async () => {
    const account = [];
    setMineTransport(async () => []);
    setLibrarySyncTransport({
      put: async (items) => {
        account.splice(0, account.length, ...items);
        return { items: account };
      },
      get: async () => ({ items: account }),
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await createLocalPrompt({ title: "本地仍在", content: "正文" });
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="sync"]').trigger("click");
    await w.get('[data-testid="sync-now"]').trigger("click");
    await flushPromises();
    expect(account.some((row) => row.payload?.title === "本地仍在")).toBe(true);
    expect(w.find('[data-testid="login-modal"]').exists()).toBe(false);
  });

  it("saves launch at login on macos", async () => {
    const w = mount(WorkbenchShell, { props: { host: "macos" } });
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-testid="launch-at-login"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("launch_at_login")).toBe("1");
    expect(w.find('[data-testid="pref-error"]').exists()).toBe(false);
  });

  it("does not claim launch at login on windows", async () => {
    const w = mount(WorkbenchShell, { props: { host: "windows" } });
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-testid="launch-at-login"]').setValue(true);
    await flushPromises();
    expect(w.get('[data-testid="pref-error"]').text()).toContain("尚未验证");
    expect(await getLocalSetting("launch_at_login")).not.toBe("1");
    expect(w.get('[data-testid="launch-at-login"]').element.checked).toBe(false);
  });

  it("shows new and paste shortcut rows", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="shortcuts"]').trigger("click");
    expect(w.get('[data-testid="new-prompt-shortcut"]').exists()).toBe(true);
    expect(w.get('[data-testid="paste-recent-shortcut"]').exists()).toBe(true);
    expect(w.text()).toContain("新建提示词");
    expect(w.text()).toContain("快速粘贴最近使用");
  });

  it("shows open directory and zip rows with existing backup actions", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="data"]').trigger("click");
    expect(w.get('[data-testid="open-library-dir"]').exists()).toBe(true);
    expect(w.get('[data-testid="export-zip"]').exists()).toBe(true);
    expect(w.get('[data-testid="auto-backup"]').exists()).toBe(true);
    expect(w.text()).toContain("导出 JSON");
    expect(w.text()).toContain("备份库文件");
    await w.get('[data-testid="export-zip"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="zip-path"]').text().length).toBeGreaterThan(0);
  });

  it("shows appearance extras including follow-system theme", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="appearance"]').trigger("click");
    const select = w.get('[data-testid="theme-select"]');
    expect(select.text()).toContain("浅色");
    expect(select.text()).toContain("深色");
    expect(select.text()).toContain("跟随系统");
    expect(w.get('[data-testid="ui-language"]').exists()).toBe(true);
    expect(w.get('[data-testid="prompt-bilingual"]').exists()).toBe(true);
    expect(w.get('[data-testid="density"]').exists()).toBe(true);
  });

  it("keeps prompt content when bilingual is turned off", async () => {
    await createLocalPrompt({ title: "双语", content: "你好 Hello" });
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="appearance"]').trigger("click");
    await w.get('[data-testid="prompt-bilingual"]').setValue(false);
    await flushPromises();
    expect(await getLocalSetting("prompt_bilingual")).toBe("0");
    const rows = await listLocalPrompts({ query: "" });
    expect(rows[0].content).toBe("你好 Hello");
  });

  it("shows model rows without sending prompt bodies", async () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="models"]').trigger("click");
    expect(w.get('[data-testid="default-model"]').exists()).toBe(true);
    expect(w.get('[data-testid="model-catalog"]').exists()).toBe(true);
    expect(w.get('[data-testid="show-model-tags"]').exists()).toBe(true);
    expect(w.get('[data-testid="variable-hints"]').exists()).toBe(true);
    expect(w.get('[data-testid="custom-models"]').exists()).toBe(true);
    await w.get('[data-testid="variable-hints"]').setValue(true);
    await w.get('[data-testid="save-models"]').trigger("click");
    await flushPromises();
    expect(fetchSpy).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });

  it("shows the current account from the existing login", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="account"]').trigger("click");
    expect(w.get('[data-testid="current-account"]').text()).toContain("dev@promptark.local");
    expect(w.text()).not.toMatch(/QQ|LinuxDo|Google/);
    await w.get('[data-testid="settings-logout"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="current-account"]').text()).toContain("未登录");
  });

  it("lists my pending publications on the account page", async () => {
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    setMineTransport(async () => [
      { id: "pub-1", source_id: "mem-1", status: "pending", title: "新稿" },
    ]);
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await flushPromises();
    const mine = w.get('[data-testid="my-publications"]');
    expect(mine.text()).toContain("新稿");
    expect(mine.text()).toContain("pending");
    expect(w.text()).not.toMatch(/QQ|LinuxDo|Google/);
  });

  it("saves author display name after login and refuses when signed out", async () => {
    const stored = { display_name: "", bio: "" };
    const puts = [];
    setMeTransport({
      get: async () => ({
        email: "dev@promptark.local",
        display_name: stored.display_name,
        bio: stored.bio,
      }),
      put: async (body) => {
        puts.push(body);
        stored.display_name = body.display_name;
        stored.bio = body.bio;
        return { email: "dev@promptark.local", ...stored };
      },
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    setMineTransport(async () => []);
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    let w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await w.get('[data-testid="author-display-name"]').setValue("林晚");
    await w.get('[data-testid="save-author-profile"]').trigger("click");
    await flushPromises();
    expect(puts[0].display_name).toBe("林晚");
    w.unmount();

    w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="author-display-name"]').element.value).toBe("林晚");
    expect(w.text()).not.toMatch(/QQ|LinuxDo|Google/);
    w.unmount();

    await logoutSession();
    puts.length = 0;
    w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    expect(w.get('[data-testid="save-author-profile"]').element.disabled).toBe(true);
    await w.get('[data-testid="save-author-profile"]').trigger("click");
    await flushPromises();
    expect(puts).toHaveLength(0);
  });

  it("does not request square when access is off", async () => {
    const squareSpy = vi.fn(async () => [{ id: "sq-1", title: "广场条目", kind: "prompt" }]);
    setSquareTransport(squareSpy);
    await setLocalSetting("square_access", "0");
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    expect(squareSpy).not.toHaveBeenCalled();
    expect(w.get('[data-testid="square-blocked"]').text()).toContain("关闭");
  });

  it("clears use history without deleting prompt content", async () => {
    const created = await createLocalPrompt({ title: "条目A", content: "中文 English" });
    await recordLocalPromptUse(created.id);
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="privacy"]').trigger("click");
    await w.get('[data-testid="clear-use-history"]').trigger("click");
    await flushPromises();
    const rows = await listLocalPrompts({ query: "" });
    expect(rows).toHaveLength(1);
    expect(rows[0].content).toBe("中文 English");
    expect(rows[0].use_count).toBe(0);
  });
});
