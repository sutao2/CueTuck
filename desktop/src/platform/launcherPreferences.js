import { getLocalSetting } from './library.js';

export const LAUNCHER_PREFERENCES_KEY = 'launcher_preferences';
export const LAUNCHER_SIZES = {
  compact: { width: 620, height: 420 },
  standard: { width: 680, height: 500 },
  large: { width: 760, height: 560 },
};
export const DEFAULT_LAUNCHER_PREFERENCES = { size: 'compact', position: 'upper', fontSize: 12, resultLimit: 20 };

export function parseLauncherPreferences(raw) {
  let value;
  try { value = JSON.parse(raw); } catch { value = null; }
  return {
    size: ['compact', 'standard', 'large'].includes(value?.size) ? value.size : 'compact',
    position: ['upper', 'center'].includes(value?.position) ? value.position : 'upper',
    fontSize: [12, 14, 16].includes(value?.fontSize) ? value.fontSize : 12,
    resultLimit: [10, 20, 50].includes(value?.resultLimit) ? value.resultLimit : 20,
  };
}

export async function readLauncherPreferences() {
  return parseLauncherPreferences(await getLocalSetting(LAUNCHER_PREFERENCES_KEY));
}
