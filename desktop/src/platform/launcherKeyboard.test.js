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

  it.each([
    ["ArrowDown", "move", 1], ["ArrowUp", "move", -1],
    ["PageDown", "move", 5], ["PageUp", "move", -5],
    ["Home", "moveTo", 0], ["End", "moveTo", 19],
  ])("routes %s and consumes only handled keys", (key, handler, value) => {
    const action = vi.fn();
    const event = keyEvent(key);
    expect(handleLauncherSearchKey(event, { rowCount: 20, [handler]: action })).toBe(true);
    expect(action).toHaveBeenCalledWith(value);
    expect(event.preventDefault).toHaveBeenCalledOnce();
    expect(event.stopPropagation).toHaveBeenCalledOnce();
  });

  it("does not activate on empty results, composition or repeated Enter", () => {
    const activate = vi.fn();
    handleLauncherSearchKey(keyEvent("Enter"), { activate });
    for (const extras of [{ isComposing: true }, { keyCode: 229 }, { repeat: true }]) {
      handleLauncherSearchKey(keyEvent("Enter", extras), { current: () => ({ id: "x" }), activate });
    }
    expect(activate).not.toHaveBeenCalled();
    const typing = keyEvent("a");
    expect(handleLauncherSearchKey(typing)).toBe(false);
    expect(typing.preventDefault).not.toHaveBeenCalled();
  });
});
