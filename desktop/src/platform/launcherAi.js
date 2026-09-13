import { invokeCommand } from './tauri.js';
function native() {
  if (!window.__TAURI_INTERNALS__) throw new Error('请在桌面客户端配置和使用 AI 优化');
}
export async function getLauncherAiConfig() { native(); return invokeCommand('get_launcher_ai_config'); }
export async function saveLauncherAiConfig(config) { native(); return invokeCommand('save_launcher_ai_config', { config }); }
export async function clearLauncherAiConfig() { native(); return invokeCommand('clear_launcher_ai_config'); }
export async function listLauncherAiModels() { native(); return invokeCommand('list_launcher_ai_models'); }
export async function optimizeLauncherPrompt(text) { native(); return invokeCommand('optimize_launcher_prompt', { text }); }
