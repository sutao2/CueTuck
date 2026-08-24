import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
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
});
