import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import UsePromptModal from "./UsePromptModal.vue";

describe("UsePromptModal", () => {
  it("asks for one variable at a time then previews the filled text", async () => {
    const w = mount(UsePromptModal, {
      props: {
        prompt: {
          id: "p-1",
          title: "行程",
          content: "去 {{城市}} 玩 {{天数}} 天，再提一次 {{城市}}",
        },
      },
    });
    expect(w.get('[data-testid="use-variable"]').text()).toBe("城市");
    expect(w.find('[data-testid="variable-hint"]').exists()).toBe(false);
    expect(w.text()).not.toContain("天数");
    await w.get('[data-testid="use-value"]').setValue("京都");
    await w.get('[data-testid="use-next"]').trigger("click");
    expect(w.get('[data-testid="use-variable"]').text()).toBe("天数");
    expect(w.text()).not.toContain("京都");
    await w.get('[data-testid="use-value"]').setValue("3");
    await w.get('[data-testid="use-next"]').trigger("click");
    expect(w.get('[data-testid="use-preview"]').text()).toBe("去 京都 玩 3 天，再提一次 京都");
    await w.get('[data-testid="use-next"]').trigger("click");
    expect(w.emitted("copied")[0][0]).toBe("去 京都 玩 3 天，再提一次 京都");
  });

  it("shows a local variable hint when hints are enabled", async () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal("fetch", fetchSpy);
    const w = mount(UsePromptModal, {
      props: {
        hintsEnabled: true,
        prompt: {
          id: "p-hint",
          title: "行程",
          content: "去 {{城市}} 玩",
        },
      },
    });
    expect(w.get('[data-testid="variable-hint"]').text()).toMatch(/京都/);
    expect(fetchSpy).not.toHaveBeenCalled();
    vi.unstubAllGlobals();
  });

  it("skips fill and previews when the prompt has no variables", async () => {
    const w = mount(UsePromptModal, {
      props: {
        prompt: {
          id: "p-2",
          title: "直出",
          content: "直接复制这段",
        },
      },
    });
    expect(w.find('[data-testid="use-variable"]').exists()).toBe(false);
    expect(w.get('[data-testid="use-preview"]').text()).toBe("直接复制这段");
    expect(w.get('[data-testid="use-next"]').text()).toContain("复制并完成");
    await w.get('[data-testid="use-next"]').trigger("click");
    expect(w.emitted("copied")[0][0]).toBe("直接复制这段");
  });

  it("advances on Enter and stays on Shift+Enter", async () => {
    const w = mount(UsePromptModal, {
      props: {
        prompt: {
          id: "p-3",
          title: "行程",
          content: "去 {{城市}} 玩 {{天数}} 天",
        },
      },
    });
    await w.get('[data-testid="use-value"]').setValue("京都");
    await w.get('[data-testid="use-value"]').trigger("keydown", { key: "Enter", shiftKey: false });
    expect(w.get('[data-testid="use-variable"]').text()).toBe("天数");
    await w.get('[data-testid="use-value"]').trigger("keydown", { key: "Enter", shiftKey: true });
    expect(w.get('[data-testid="use-variable"]').text()).toBe("天数");
    expect(w.find('[data-testid="use-preview"]').exists()).toBe(false);
  });
});
