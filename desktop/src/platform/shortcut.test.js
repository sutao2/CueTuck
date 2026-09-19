import { describe, expect, it, vi } from "vitest";
import { registerLauncherShortcut, setShortcutRecording, shortcutStatus } from "./shortcut.js";

describe("registerLauncherShortcut", () => {
  it('reports startup failures and clears the warning only after registration succeeds', async () => {
    const options = { register: vi.fn(), unregisterAll: vi.fn().mockRejectedValueOnce(Error('系统拒绝')), persist: vi.fn() };
    await expect(registerLauncherShortcut('Control+Space', options)).rejects.toThrow('系统拒绝');
    expect(shortcutStatus.error).toBe('系统拒绝');
    expect(options.persist).not.toHaveBeenCalled();
    await registerLauncherShortcut('Control+Space', options);
    expect(shortcutStatus.error).toBe('');
  });
  it("suppresses all application shortcuts during recording and restores callbacks afterward", async () => {
    const callbacks = [];
    const extra = vi.fn();
    await registerLauncherShortcut("Control+Space", {
      register: vi.fn(async (_, callback) => { callbacks.push(callback); }),
      unregisterAll: vi.fn(), persist: vi.fn(),
      extras: [{ combo: "Control+Alt+N", handler: extra }],
    });
    setShortcutRecording(true);
    try {
      await callbacks[0]({ state: "Pressed" });
      await callbacks[1]({ state: "Pressed" });
      expect(extra).not.toHaveBeenCalled();
    } finally { setShortcutRecording(false); }
    await callbacks[1]({ state: "Pressed" });
    expect(extra).toHaveBeenCalledOnce();
  });
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
