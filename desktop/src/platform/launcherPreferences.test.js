import { beforeEach, expect, it, vi } from 'vitest';
import { resetMemoryLibrary, setLocalSetting } from './library.js';
import { DEFAULT_LAUNCHER_PREFERENCES, LAUNCHER_PREFERENCES_KEY, parseLauncherPreferences, readLauncherPreferences } from './launcherPreferences.js';
import { openLauncherWindow } from './launcherWindow.js';

beforeEach(resetMemoryLibrary);

it('defaults missing and malformed preferences without accepting out-of-range values', () => {
  for (const raw of ['', '{bad', 'null', '[]', '0', '{"size":"__proto__","position":"bottom","fontSize":99,"resultLimit":999}']) {
    expect(parseLauncherPreferences(raw)).toEqual(DEFAULT_LAUNCHER_PREFERENCES);
  }
  expect(parseLauncherPreferences('{"size":"large","position":"center","fontSize":16,"resultLimit":10}'))
    .toEqual({ size: 'large', position: 'center', fontSize: 16, resultLimit: 10 });
  expect(parseLauncherPreferences('{"size":"standard","fontSize":"16"}')).toEqual({ ...DEFAULT_LAUNCHER_PREFERENCES, size: 'standard' });
});

it('loads local preferences and opens browser preview at the chosen size', async () => {
  await setLocalSetting(LAUNCHER_PREFERENCES_KEY, JSON.stringify({ size: 'large' }));
  expect((await readLauncherPreferences()).size).toBe('large');
  const popup = vi.spyOn(window, 'open').mockReturnValue({});
  try {
    await openLauncherWindow();
    expect(popup).toHaveBeenCalledExactlyOnceWith('/launcher.html', 'launcher', 'width=760,height=560');
  } finally { popup.mockRestore(); }
});
