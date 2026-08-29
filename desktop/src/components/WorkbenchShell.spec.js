import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, it, expect, vi } from "vitest";
import WorkbenchShell from "./WorkbenchShell.vue";
import SettingsModal from "./SettingsModal.vue";
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
import { resetUpdates, setUpdateTransport, setInstallTransport } from "../platform/updates.js";
import { resetBilling, setBillingTransport } from "../platform/billing.js";
import {
  resetSquare,
  setFavoriteTransport,
  setMineTransport,
  setPublishTransport,
  setDownloadStatsTransport,
  setSquareContentTransport,
  setSquareTransport,
} from "../platform/square.js";
import { listSyncQueue } from "../platform/syncQueue.js";

const tauriVersion = JSON.parse(
  readFileSync(
    resolve(dirname(fileURLToPath(import.meta.url)), "../../src-tauri/tauri.conf.json"),
    "utf8",
  ),
).version;

describe("WorkbenchShell", () => {
  beforeEach(() => {
    resetMemoryLibrary();
    resetMemorySession();
    resetLibrarySync();
    resetUpdates();
    resetBilling();
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

  it("does not record anonymous download stats when the setting is off", async () => {
    const calls = [];
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    setDownloadStatsTransport(async (request) => {
      calls.push(request);
    });
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="download-square"]').trigger("click");
    await flushPromises();
    expect(calls).toEqual([]);
    const rows = await listLocalPrompts({ query: "自然光群像" });
    expect(rows).toHaveLength(1);
  });

  it("records anonymous download stats after a successful download when the setting is on", async () => {
    const calls = [];
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSquareContentTransport(async (id) => ({
      id,
      title: "自然光群像",
      content: "清透蓝天下的多元人物群像。",
    }));
    setDownloadStatsTransport(async (request) => {
      calls.push(request);
    });
    await setLocalSetting("anonymous_download_stats", "1");
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="download-square"]').trigger("click");
    await flushPromises();
    expect(calls).toEqual([
      {
        id: "sq-1",
        method: "POST",
        path: "/v1/square/items/sq-1/downloads",
        headers: {},
      },
    ]);
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
    expect(w.get('[data-testid="auto-sync-queue-row"]').text()).not.toContain("尚未提供");
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
    expect(panel.text()).toContain(`桌面包 ${tauriVersion}`);
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

  it("queues an updater install when auto-download is on and the channel has a package", async () => {
    setUpdateTransport(async ({ channel } = {}) => ({
      available: true,
      notes: "preview notes",
      version: "0.2.0-beta",
      channel: channel ?? "stable",
    }));
    setInstallTransport(async ({ channel }) => ({
      queued: true,
      via: "updater",
      channel,
    }));
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="updates"]').trigger("click");
    await w.get('[data-testid="auto-download"]').setValue(true);
    await w.get('[data-testid="update-channel"]').setValue("preview");
    await flushPromises();
    expect(await getLocalSetting("auto_download")).toBe("1");
    expect(await getLocalSetting("update_channel")).toBe("preview");
    await w.get('[data-testid="check-updates"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="update-note"]').text()).toContain("已排队安装");
    expect(w.get('[data-testid="release-notes"]').text()).toContain("preview notes");
    expect(w.get('[data-testid="settings-updates"]').text()).not.toMatch(/已从商店|Microsoft Store|Mac App Store/);
  });

  it("shows sync rows without requesting the backend", async () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="sync"]').trigger("click");
    const panel = w.get('[data-testid="settings-unavailable"]');
    expect(panel.text()).not.toContain("尚未提供");
    expect(panel.text()).toContain("自动同步收藏");
    expect(w.get('[data-testid="auto-sync-queue"]').element.checked).toBe(false);
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

  it("labels conflict handling as newer-wins instead of unavailable", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="sync"]').trigger("click");
    const row = w.get('[data-testid="sync-conflict"]');
    expect(row.text()).toContain("冲突处理");
    expect(row.text()).toContain("较新者胜");
    expect(row.text()).not.toContain("尚未提供");
    expect(w.get('[data-testid="sync-conflict-strategy"]').element.value).toBe("newer");
  });

  it("keeps the local body when keep-local is selected before syncing", async () => {
    const { insertSyncedLocalPrompt } = await import("../platform/library.js");
    setMineTransport(async () => []);
    setLibrarySyncTransport({
      put: async (items) => ({ items }),
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
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await insertSyncedLocalPrompt({
      id: "p-1",
      title: "本地仍在",
      content: "本机正文",
      updatedAt: "1",
    });
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="sync"]').trigger("click");
    await w.get('[data-testid="sync-conflict-strategy"]').setValue("keep_local");
    await flushPromises();
    expect(await getLocalSetting("sync_conflict")).toBe("keep_local");
    await w.get('[data-testid="sync-now"]').trigger("click");
    await flushPromises();
    const rows = await listLocalPrompts({ query: "本地仍在" });
    expect(rows[0].content).toBe("本机正文");
    w.unmount();
    const again = mount(WorkbenchShell);
    await again.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await again.get('[data-settings-page="sync"]').trigger("click");
    expect(again.get('[data-testid="sync-conflict-strategy"]').element.value).toBe("keep_local");
  });

  it("does not claim the network page has no cloud sync", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="network"]').trigger("click");
    const row = w.get('[data-testid="sync-status"]');
    expect(row.text()).toContain("同步状态");
    expect(row.text()).toContain("手动立即同步");
    expect(row.text()).not.toContain("没有云同步");
    expect(row.text()).not.toContain("尚未提供");
    expect(row.text()).not.toMatch(/已同步|正在同步/);
  });

  it("does not claim Wi-Fi image sync is missing because there is no cloud engine", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="sync"]').trigger("click");
    const row = w.get('[data-testid="sync-wifi-images-row"]');
    expect(row.text()).toContain("仅在 Wi-Fi");
    expect(row.text()).not.toContain("尚未提供");
    expect(row.text()).not.toContain("没有云同步");
    expect(w.get('[data-testid="sync-wifi-images"]').element.checked).toBe(false);
  });

  it("persists wifi-only image sync from the settings row", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="sync"]').trigger("click");
    await w.get('[data-testid="sync-wifi-images"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("sync_wifi_images")).toBe("1");
    w.unmount();
    const again = mount(WorkbenchShell);
    await again.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await again.get('[data-settings-page="sync"]').trigger("click");
    expect(again.get('[data-testid="sync-wifi-images"]').element.checked).toBe(true);
  });

  it("persists auto-sync queue from the settings row", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="sync"]').trigger("click");
    await w.get('[data-testid="auto-sync-queue"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("auto_sync_queue")).toBe("1");
    w.unmount();
    const again = mount(WorkbenchShell);
    await again.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await again.get('[data-settings-page="sync"]').trigger("click");
    expect(again.get('[data-testid="auto-sync-queue"]').element.checked).toBe(true);
  });

  it("does not claim anonymous download stats is unavailable", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="privacy"]').trigger("click");
    const row = w.get('[data-testid="anonymous-download-stats-row"]');
    expect(row.text()).toContain("匿名下载统计");
    expect(row.text()).toContain("条目 id");
    expect(row.text()).not.toContain("尚未提供");
    expect(w.get('[data-testid="anonymous-download-stats"]').element.checked).toBe(false);
  });

  it("persists anonymous download stats from the settings row", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="privacy"]').trigger("click");
    await w.get('[data-testid="anonymous-download-stats"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("anonymous_download_stats")).toBe("1");
    w.unmount();
    const again = mount(WorkbenchShell);
    await again.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await again.get('[data-settings-page="privacy"]').trigger("click");
    expect(again.get('[data-testid="anonymous-download-stats"]').element.checked).toBe(true);
  });

  it("labels the proxy row as follow-system instead of available", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="network"]').trigger("click");
    const row = w.get('[data-testid="proxy-row"]');
    expect(row.text()).toContain("代理");
    expect(row.text()).toContain("跟随系统");
    expect(row.text()).not.toContain("尚未提供");
    expect(row.text()).toContain("浏览器预览不走该代理");
    expect(w.get('[data-testid="http-proxy"]').element.value).toBe("");
  });

  it("persists a manual http proxy from the settings row", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="network"]').trigger("click");
    const input = w.get('[data-testid="http-proxy"]');
    await input.setValue("http://127.0.0.1:7890");
    await input.trigger("change");
    await flushPromises();
    expect(await getLocalSetting("http_proxy")).toBe("http://127.0.0.1:7890");
    w.unmount();
    const again = mount(WorkbenchShell);
    await again.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await again.get('[data-settings-page="network"]').trigger("click");
    expect(again.get('[data-testid="http-proxy"]').element.value).toBe("http://127.0.0.1:7890");
  });

  it("rejects an invalid proxy url without saving", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="network"]').trigger("click");
    const input = w.get('[data-testid="http-proxy"]');
    await input.setValue("not-a-url");
    await input.trigger("change");
    await flushPromises();
    expect(w.get('[data-testid="proxy-error"]').text()).toContain("代理地址无效");
    expect(await getLocalSetting("http_proxy")).toBe("");
  });

  it("queues a favorite while offline when auto-sync is on and flushes on sync now", async () => {
    setSquareTransport(async () => [{ id: "sq-1", title: "自然光群像", kind: "prompt" }]);
    setSessionTransport(async () => ({
      access_token: "acc.1",
      refresh_token: "ref.1",
      email: "dev@promptark.local",
    }));
    await loginSession({ email: "dev@promptark.local", password: "devpass" });
    await setLocalSetting("auto_sync_queue", "1");
    setFavoriteTransport(async (request) => {
      if (request.method === "GET") return { items: [] };
      throw new Error("收藏失败");
    });
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="favorite-square"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="favorite-square"]').text()).toBe("已收藏");
    expect(await listSyncQueue()).toEqual([
      { kind: "favorite", method: "PUT", id: "sq-1", email: "dev@promptark.local" },
    ]);
    expect(await listLocalPrompts({ query: "" })).toHaveLength(0);
    expect(w.find('[data-testid="square-offline"]').exists()).toBe(false);
    const flushed = [];
    setFavoriteTransport(async (request) => {
      flushed.push(request);
      if (request.method === "GET") return { items: flushed.some((call) => call.method === "PUT") ? [{ id: "sq-1" }] : [] };
      return { id: request.id };
    });
    setLibrarySyncTransport({
      put: async (items) => ({ items }),
      get: async () => ({ items: [] }),
    });
    setMineTransport(async () => []);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="sync"]').trigger("click");
    await w.get('[data-testid="sync-now"]').trigger("click");
    await flushPromises();
    expect(flushed.some((call) => call.method === "PUT" && call.id === "sq-1")).toBe(true);
    expect(await listSyncQueue()).toEqual([]);
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

  it("saves launch at login and tray on windows without claiming nsis", async () => {
    const w = mount(WorkbenchShell, { props: { host: "windows" } });
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-testid="launch-at-login"]').setValue(true);
    await w.get('[data-testid="minimize-to-tray"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("launch_at_login")).toBe("1");
    expect(await getLocalSetting("minimize_to_tray")).toBe("1");
    expect(w.find('[data-testid="pref-error"]').exists()).toBe(false);
    expect(w.text()).not.toMatch(/NSIS 已验证|Windows 已验证/);
  });

  it("saves launch at login and tray on linux without claiming release qa", async () => {
    const w = mount(WorkbenchShell, { props: { host: "linux" } });
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-testid="launch-at-login"]').setValue(true);
    await w.get('[data-testid="minimize-to-tray"]').setValue(true);
    await flushPromises();
    expect(await getLocalSetting("launch_at_login")).toBe("1");
    expect(await getLocalSetting("minimize_to_tray")).toBe("1");
    expect(w.find('[data-testid="pref-error"]').exists()).toBe(false);
    expect(w.text()).not.toMatch(/Linux 已验证|发行 QA 已通过|NSIS 已验证/);
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
    await flushPromises();
    await w.get('[data-sort="最近"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("条目A");
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="privacy"]').trigger("click");
    await w.get('[data-testid="clear-use-history"]').trigger("click");
    await flushPromises();
    const rows = await listLocalPrompts({ query: "" });
    expect(rows).toHaveLength(1);
    expect(rows[0].content).toBe("中文 English");
    expect(rows[0].use_count).toBe(0);
    expect(rows[0].last_used_at).toBeFalsy();
    await w.get(".modal-close").trigger("click");
    await flushPromises();
    expect(w.find('[data-testid="library-view"]').exists()).toBe(false);
    expect(w.text()).toContain("还没有最近使用");
  });

  it("does not claim the keychain row uses the local keychain in browser preview", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="privacy"]').trigger("click");
    const row = w.get('[data-testid="keychain-row"]');
    expect(row.text()).toContain("系统钥匙串");
    expect(row.text()).toContain("不进 Web Storage");
    expect(row.text()).not.toContain("本机钥匙串");
    expect(w.text()).toContain("匿名下载统计");
    expect(w.get('[data-testid="anonymous-download-stats-row"]').text()).not.toContain("尚未提供");
  });

  it("does not claim login writes refresh to the system keychain in browser preview", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await w.get('[data-settings-page="account"]').trigger("click");
    await w.get('[data-testid="settings-login"]').trigger("click");
    expect(w.get('[data-testid="login-token-note"]').text()).not.toContain("只写入系统钥匙串");
    expect(w.get('[data-testid="login-token-note"]').text()).toContain("不进 Web Storage");
  });

  it("labels the keychain row as local keychain inside Tauri", async () => {
    const w = mount(SettingsModal, {
      props: { session: { loggedIn: false, email: "" }, host: "macos", theme: "light" },
    });
    window.__TAURI_INTERNALS__ = {};
    try {
      await w.get('[data-settings-page="privacy"]').trigger("click");
      expect(w.get('[data-testid="keychain-row"]').text()).toContain("本机钥匙串");
      expect(w.get('[data-testid="keychain-row"]').text()).toContain("不进 Web Storage");
    } finally {
      delete window.__TAURI_INTERNALS__;
    }
  });

  it("shows unpaid billing as 支付未开通 and does not open checkout", async () => {
    const opened = [];
    vi.stubGlobal("open", (url) => {
      opened.push(url);
      return null;
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    setMineTransport(async () => []);
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
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="billing-note"]').text()).toContain("支付未开通");
    expect(w.get('[data-testid="billing-pro"]').text()).toContain("未订阅");
    await w.get('[data-testid="billing-checkout"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="billing-note"]').text()).toContain("支付未开通");
    expect(opened).toEqual([]);
    expect(w.get('[data-testid="settings-modal"]').text()).not.toMatch(/已从商店|已经上架/);
    vi.unstubAllGlobals();
  });

  it("opens Stripe checkout only when a test checkout url is returned", async () => {
    const opened = [];
    vi.stubGlobal("open", (url) => {
      opened.push(url);
      return null;
    });
    setSessionTransport(async () => ({ email: "dev@promptark.local", access_token: "tok" }));
    setMineTransport(async () => []);
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
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    await flushPromises();
    await w.get('[data-settings-page="account"]').trigger("click");
    await flushPromises();
    await w.get('[data-testid="billing-checkout"]').trigger("click");
    await flushPromises();
    expect(opened).toEqual(["https://checkout.stripe.com/c/pay/cs_test_preview"]);
    expect(w.get('[data-testid="billing-pro"]').text()).toContain("未订阅");
    expect(w.get('[data-testid="settings-modal"]').text()).not.toMatch(/已从商店|已经上架/);
    vi.unstubAllGlobals();
  });

  it("shows recently used local prompts on the recent tab", async () => {
    await createLocalPrompt({ title: "未用过", content: "a" });
    const used = await createLocalPrompt({ title: "刚用过", content: "b" });
    await recordLocalPromptUse(used.id);
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get('[data-sort="最近"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("刚用过");
    expect(w.get('[data-testid="library-view"]').text()).not.toContain("未用过");
  });

  it("shows only starred local prompts on the favorite tab", async () => {
    await createLocalPrompt({ title: "星标条目", content: "a" });
    await createLocalPrompt({ title: "普通条目", content: "b" });
    const w = mount(WorkbenchShell);
    await flushPromises();
    const card = w.findAll(".prompt-card").find((row) => row.text().includes("星标条目"));
    await card.trigger("contextmenu", { clientX: 20, clientY: 20 });
    await w.get('[data-testid="context-menu"] [data-action="favorite"]').trigger("click");
    await flushPromises();
    await w.get('[data-sort="收藏"]').trigger("click");
    await flushPromises();
    expect(w.get('[data-testid="library-view"]').text()).toContain("星标条目");
    expect(w.get('[data-testid="library-view"]').text()).not.toContain("普通条目");
  });

  it("opens a context menu with existing local actions", async () => {
    await createLocalPrompt({ title: "可编辑", content: "x" });
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get(".prompt-card").trigger("contextmenu", { clientX: 40, clientY: 80 });
    const menu = w.get('[data-testid="context-menu"]');
    expect(menu.text()).toContain("编辑");
    expect(menu.text()).toContain("使用");
    expect(menu.text()).toContain("收藏");
    expect(menu.text()).toContain("删除");
    expect(menu.text()).not.toContain("举报");
    expect(menu.text()).not.toContain("分享");
    await menu.get('[data-action="edit"]').trigger("click");
    expect(w.get('[data-testid="prompt-editor"]').exists()).toBe(true);
  });

  it("shows a local variable hint from the workbench setting", async () => {
    await createLocalPrompt({ title: "行程", content: "去 {{城市}} 玩" });
    await setLocalSetting("variable_hints", "1");
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get(".card-action").trigger("click");
    expect(w.get('[data-testid="variable-hint"]').text()).toMatch(/京都/);
  });

  it("shows model tags on cards when the setting is on", async () => {
    await createLocalPrompt({ title: "Flux 片", content: "x", model: "Flux" });
    await setLocalSetting("show_model_tags", "1");
    const w = mount(WorkbenchShell);
    await flushPromises();
    expect(w.get('[data-testid="model-tag"]').text()).toBe("Flux");
  });

  it("hides model tags when the setting is off", async () => {
    await createLocalPrompt({ title: "Flux 片", content: "x", model: "Flux" });
    await setLocalSetting("show_model_tags", "0");
    const w = mount(WorkbenchShell);
    await flushPromises();
    expect(w.find('[data-testid="model-tag"]').exists()).toBe(false);
  });

  it("does not apply the default model when editing an untagged prompt", async () => {
    await createLocalPrompt({ title: "无模型", content: "x" });
    await setLocalSetting("default_model", "Flux");
    await setLocalSetting("model_catalog", "Flux");
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get(".prompt-card").trigger("click");
    expect(w.get('[data-testid="prompt-model"]').element.value).toBe("");
  });

  it("preselects the default model in the editor", async () => {
    await setLocalSetting("default_model", "Flux");
    await setLocalSetting("model_catalog", "Flux\nGPT-5");
    const w = mount(WorkbenchShell);
    await flushPromises();
    await w.get(".content-actions .primary-button").trigger("click");
    const select = w.get('[data-testid="prompt-model"]');
    expect(select.findAll("option").map((option) => option.element.value)).toEqual(["", "Flux", "GPT-5"]);
    expect(select.element.value).toBe("Flux");
  });

  it("filters square items by the selected model", async () => {
    const seen = [];
    setSquareTransport(async (request) => {
      seen.push(request);
      return [
        { id: "a", title: "Flux 人像", kind: "prompt", model: "Flux" },
        { id: "b", title: "GPT 文案", kind: "prompt", model: "GPT-5" },
      ];
    });
    await setLocalSetting("model_catalog", "Flux\nGPT-5");
    const w = mount(WorkbenchShell);
    await w.get('[data-space="square"]').trigger("click");
    await flushPromises();
    const select = w.get('[data-testid="model-filter"]');
    expect(select.findAll("option").map((option) => option.element.value)).toEqual(["", "Flux", "GPT-5"]);
    await select.setValue("Flux");
    await flushPromises();
    expect(seen.at(-1).model).toBe("Flux");
  });
});
