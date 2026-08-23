import { describe, expect, it, vi } from "vitest";
import { registerLauncherShortcut } from "./shortcut.js";

describe("registerLauncherShortcut", () => {
  it("does not persist when register throws", async () => {
    const persist = vi.fn();
    await expect(
      registerLauncherShortcut("Control+Space", {
        register: vi.fn().mockRejectedValue(new Error("already registered")),
        unregisterAll: vi.fn().mockResolvedValue(undefined),
        persist,
      }),
    ).rejects.toThrow(/already registered|冲突/);
    expect(persist).not.toHaveBeenCalled();
  });

  it("persists after a successful register", async () => {
    const persist = vi.fn();
    await registerLauncherShortcut("Control+Space", {
      register: vi.fn().mockResolvedValue(undefined),
      unregisterAll: vi.fn().mockResolvedValue(undefined),
      persist,
    });
    expect(persist).toHaveBeenCalledWith("Control+Space");
  });

  it("does not persist when an extra shortcut register throws", async () => {
    const persist = vi.fn();
    const register = vi
      .fn()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("already registered"));
    await expect(
      registerLauncherShortcut("Control+Space", {
        register,
        unregisterAll: vi.fn().mockResolvedValue(undefined),
        persist,
        extras: [
          {
            combo: "Control+Alt+N",
            handler: vi.fn(),
          },
        ],
      }),
    ).rejects.toThrow(/already registered|冲突/);
    expect(persist).not.toHaveBeenCalled();
  });
});
