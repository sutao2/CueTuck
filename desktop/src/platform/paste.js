export async function copyLauncherText(text) {
  if (window.__TAURI_INTERNALS__ && /Mac/.test(navigator.userAgent)) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke("copy_launcher_text", { text });
  }
  return navigator.clipboard.writeText(text);
}

export async function copyThenPaste(
  text,
  { writeText = copyLauncherText, invoke } = {},
) {
  await writeText(text);
  const run = invoke ?? (async (command) => {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke(command);
  });
  try {
    await run("paste_to_active_app");
    return { ok: true };
  } catch (error) {
    return { ok: false, message: `已复制，未能粘贴：${error?.message || error}` };
  }
}
