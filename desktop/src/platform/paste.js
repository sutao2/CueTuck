export async function copyThenPaste(
  text,
  { writeText = (value) => navigator.clipboard.writeText(value), invoke } = {},
) {
  await writeText(text);
  const run = invoke ?? (async (command) => {
    const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
    return tauriInvoke(command);
  });
  try {
    await run("paste_to_active_app");
    return { ok: true };
  } catch {
    return { ok: false, message: "已复制，未能粘贴" };
  }
}
