import { describe, expect, it, vi } from "vitest";
import { handleLauncherSearchKey } from "./launcherKeyboard.js";

function keyEvent(key, extras = {}) {
  return {
    key,
    metaKey: false,
    ctrlKey: false,
    isComposing: false,
    preventDefault: vi.fn(),
    stopPropagation: vi.fn(),
    ...extras,
  };
}

describe("handleLauncherSearchKey", () => {
  it("activates default on Enter and copy on Ctrl+Enter", () => {
    const activate = vi.fn();
    const row = { id: "1", title: "官网" };
    handleLauncherSearchKey(keyEvent("Enter"), {
      current: () => row,
      activate,
    });
    expect(activate).toHaveBeenCalledWith(row, "default");
    handleLauncherSearchKey(keyEvent("Enter", { ctrlKey: true }), {
      current: () => row,
      activate,
    });
    expect(activate).toHaveBeenCalledWith(row, "copy");
  });

  it("closes on Escape", () => {
    const close = vi.fn();
    handleLauncherSearchKey(keyEvent("Escape"), { close });
    expect(close).toHaveBeenCalled();
  });
});
