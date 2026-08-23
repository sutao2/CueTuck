import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, it, expect } from "vitest";
import WorkbenchShell from "./WorkbenchShell.vue";
import { createLocalCollection, createLocalPrompt, resetMemoryLibrary } from "../platform/library.js";
import { resetMemorySession, setSessionTransport } from "../platform/session.js";
import { resetSquare, setSquareTransport } from "../platform/square.js";

describe("WorkbenchShell", () => {
  beforeEach(() => {
    resetMemoryLibrary();
    resetMemorySession();
    resetSquare();
    setSquareTransport(async () => {
      throw new Error("广场暂时不可用");
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

  it("opens settings from the sidebar", async () => {
    const w = mount(WorkbenchShell);
    await w.get('[data-testid="open-settings"]').trigger("click");
    expect(w.get('[data-testid="settings-modal"]').exists()).toBe(true);
    await w.get('[data-settings-page="sync"]').trigger("click");
    expect(w.get('[data-testid="settings-unavailable"]').text()).toContain("不会请求网络");
  });
});
