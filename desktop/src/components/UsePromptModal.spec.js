import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import UsePromptModal from "./UsePromptModal.vue";

describe("UsePromptModal", () => {
  it('prefills imported defaults, preserves edits on back and copies resolved arguments',async()=>{
    const w=mount(UsePromptModal,{props:{prompt:{title:'广告',content:'{argument name="aspect ratio" default="21:9 全景"} / {argument name="background color" default="产品主色调"}'}}});
    expect(w.get('[data-testid="use-value"]').element.value).toBe('21:9 全景');
    await w.get('[data-testid="use-value"]').setValue('16:9');await w.get('[data-testid="use-next"]').trigger('click');
    expect(w.get('[data-testid="use-value"]').element.value).toBe('产品主色调');
    await w.get('.ghost-button').trigger('click');expect(w.get('[data-testid="use-value"]').element.value).toBe('16:9');
    await w.get('[data-testid="use-next"]').trigger('click');
    await w.get('[data-testid="use-next"]').trigger('click');expect(w.get('[data-testid="use-preview"]').text()).toBe('16:9 / 产品主色调');
    await w.get('[data-testid="use-next"]').trigger('click');expect(w.emitted('copied')[0][0]).toBe('16:9 / 产品主色调');w.unmount();
  });
  it("fills anonymous slots independently in the main-window wizard", async () => {
    const w = mount(UsePromptModal, {
      props: { prompt: { title: "Mysql", content: "Sql {} dejk fer {}. hdjjf dev {} jhdfhk sd" } },
    });
    for (const [index, value] of ["A", "B", "C"].entries()) {
      expect(w.get('[data-testid="use-variable"]').text()).toBe(`占位符 ${index + 1}`);
      await w.get('[data-testid="use-value"]').setValue(value);
      await w.get('[data-testid="use-next"]').trigger("click");
    }
    expect(w.get('[data-testid="use-preview"]').text()).toBe("Sql A dejk fer B. hdjjf dev C jhdfhk sd");
    await w.get('[data-testid="use-next"]').trigger("click");
    expect(w.emitted("copied")[0][0]).toBe("Sql A dejk fer B. hdjjf dev C jhdfhk sd");
    w.unmount();
  });
  it('keeps keyboard focus inside when the last variable becomes a preview', async () => {
    const w = mount(UsePromptModal, { attachTo: document.body, props: { prompt: { title: 'Focus', content: '{{产品}}' } } });
    await flushPromises();
    w.get('[data-testid="use-value"]').element.focus();
    await w.get('[data-testid="use-value"]').trigger('keydown', { key: 'Enter' });
    expect(document.activeElement).toBe(w.get('[data-testid="use-next"]').element);
    await w.get('[data-testid="use-next"]').trigger('keydown', { key: 'Escape' });
    expect(w.emitted('cancel')).toHaveLength(1);
    w.unmount();
  });
  it('does not advance while composing Chinese or repeat copying while busy', async () => {
    const w = mount(UsePromptModal, { props: { prompt: { title: '输入法', content: '{{主题}}' } } });
    await w.get('[data-testid="use-value"]').setValue('设计');
    await w.get('[data-testid="use-value"]').trigger('keydown', { key: 'Enter', isComposing: true });
    expect(w.find('[data-testid="use-preview"]').exists()).toBe(false);
    await w.get('[data-testid="use-value"]').trigger('keydown', { key: 'Enter' });
    expect(w.get('[data-testid="use-preview"]').text()).toBe('设计');
    await w.setProps({ busy: true });
    await w.get('[data-testid="use-next"]').trigger('click');
    expect(w.emitted('copied')).toBeUndefined();
    expect(w.get('[data-testid="use-next"]').text()).toBe('正在复制…');
    w.unmount();
  });
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
