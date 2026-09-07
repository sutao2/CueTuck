import { mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";
import ShortcutInput from "./ShortcutInput.vue";
import { setShortcutRecording } from "../platform/shortcut.js";

let w;
afterEach(() => { w?.unmount(); setShortcutRecording(false); vi.restoreAllMocks(); });
const open = () => (w = mount(ShortcutInput, { props: { modelValue: "Control+Space", host: "macos" } }));

describe("shortcut recorder", () => {
  it("records physical combinations including shifted and Option-produced characters", async () => {
    open();
    await w.trigger("focus");
    await w.trigger("keydown", { key: "˜", code: "KeyN", ctrlKey: true, altKey: true, shiftKey: true });
    expect(w.emitted("update:modelValue")[0]).toEqual(["Control+Alt+Shift+N"]);
    await w.setProps({ modelValue: "Control+Alt+Shift+N" });
    expect(w.element.value).toBe("⌃⌥⇧N");
    await w.trigger("keydown", { key: " ", code: "Space", metaKey: true });
    expect(w.emitted("update:modelValue")[1]).toEqual(["Super+Space"]);
  });

  it("ignores modifiers, IME, repeated presses and bare typing; accepts function keys", async () => {
    open();
    for (const event of [
      { key: "Control", code: "ControlLeft", ctrlKey: true },
      { key: "a", code: "KeyA" },
      { key: "A", code: "KeyA", shiftKey: true },
      { key: "a", code: "KeyA", ctrlKey: true, isComposing: true },
      { key: "a", code: "KeyA", ctrlKey: true, keyCode: 229 },
      { key: "a", code: "KeyA", ctrlKey: true, repeat: true },
    ]) await w.trigger("keydown", event);
    expect(w.emitted("update:modelValue")).toBeUndefined();
    await w.trigger("keydown", { key: "F8", code: "F8" });
    expect(w.emitted("update:modelValue")[0]).toEqual(["F8"]);
  });

  it("lets Tab navigate and Escape cancel without reaching the settings dialog", async () => {
    open();
    await w.trigger("focus");
    await w.setProps({ modelValue: "Super+K" });
    const escape = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
    w.element.dispatchEvent(escape);
    expect(escape.defaultPrevented).toBe(true);
    expect(w.emitted("update:modelValue")[0]).toEqual(["Control+Space"]);
    const tab = new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, bubbles: true, cancelable: true });
    w.element.dispatchEvent(tab);
    expect(tab.defaultPrevented).toBe(false);
    expect(w.attributes("readonly")).toBeDefined();
  });
});
